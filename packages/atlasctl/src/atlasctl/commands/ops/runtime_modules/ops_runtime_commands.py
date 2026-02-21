def _ops_policy_audit(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    env_schema = json.loads((repo / "configs/ops/env.schema.json").read_text(encoding="utf-8"))
    vars_declared = sorted(env_schema.get("variables", {}).keys())
    search_roots = [
        repo / "makefiles",
        repo / "ops",
        repo / "packages/atlasctl/src",
        repo / "crates/bijux-atlas-server/src",
    ]
    search_paths: list[Path] = []
    for root in search_roots:
        if not root.exists():
            continue
        search_paths.extend(p for p in root.rglob("*") if p.is_file() and p.suffix in {".mk", ".sh", ".py", ".rs", ".json", ".md"})
    text = "\n".join(p.read_text(encoding="utf-8", errors="ignore") for p in search_paths)
    declared_only = {"PREREQS_OK", "OPS_SMOKE_BUDGET_EXEMPTION_ID"}
    violations: list[str] = []
    for var in vars_declared:
        if var in declared_only:
            continue
        if re.search(rf"\b{re.escape(var)}\b", text) is None:
            violations.append(f"ops env variable `{var}` not reflected in make/scripts usage")
    if "configs/ops/tool-versions.json" not in (repo / "makefiles/ops.mk").read_text(encoding="utf-8"):
        violations.append("ops.mk must reference configs/ops/tool-versions.json")

    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "pass" if not violations else "fail",
        "violations": violations,
    }
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        if violations:
            for v in violations:
                print(f"ops-policy-audit violation: {v}")
        else:
            print("ops policy audit passed")
    return 0 if not violations else 1


from .ops_k8s import (
    _k8s_checks_layout,
    _k8s_flakes,
    _k8s_test_contract,
    _k8s_test_lib,
    _k8s_surface_generate,
)


def _ops_clean_generated(ctx: RunContext, report_format: str, force: bool) -> int:
    generated_root = ctx.repo_root / "ops" / "_generated"
    if not generated_root.exists():
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "run_id": ctx.run_id,
            "status": "pass",
            "message": "ops/_generated does not exist",
        }
        if report_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(payload["message"])
        return 0

    probe = run_command(["git", "check-ignore", "-q", "ops/_generated/probe.file"], ctx.repo_root)
    ignored = probe.code == 0
    if not ignored and not force:
        message = "refusing to clean ops/_generated because it is not ignored; pass --force to override"
        if report_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "bijux-atlas",
                        "run_id": ctx.run_id,
                        "status": "fail",
                        "message": message,
                    },
                    sort_keys=True,
                )
            )
        else:
            print(message)
        return 1

    removed: list[str] = []
    for child in sorted(generated_root.iterdir()):
        removed.append(child.name)
        if child.is_dir():
            shutil.rmtree(child)
        else:
            child.unlink()
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "pass",
        "path": str(generated_root.relative_to(ctx.repo_root)),
        "removed_entries": removed,
    }
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"cleaned {payload['path']} ({len(removed)} entries removed)")
    return 0


def run_ops_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.ops_cmd == "check":
        steps = [
            [*SELF_CLI, "ops", "lint", "--report", ns.report, "--emit-artifacts"],
            [*SELF_CLI, "ops", "contracts-check", "--report", ns.report],
            [*SELF_CLI, "ops", "suites-check", "--report", ns.report],
            [*SELF_CLI, "ops", "schema-check", "--report", ns.report],
            ["env", "CACHE_STATUS_STRICT=0", "make", "-s", "ops-cache-status"],
            ["make", "-s", "pins/check"],
            [*SELF_CLI, "ops", "surface", "--report", ns.report],
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_index_surface.py"],
        ]
        for cmd in steps:
            code, output = _run_check(cmd, ctx.repo_root)
            if output:
                print(output)
            if code != 0:
                return code
        return 0

    if ns.ops_cmd == "lint":
        if ns.fix:
            for cmd in (
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_surface_meta.py"],
                [*SELF_CLI, "docs", "generate", "--report", "text"],
            ):
                code, output = _run_check(cmd, ctx.repo_root)
                if code != 0:
                    if output:
                        print(output)
                    return code
        return _run_checks(
            ctx,
            checks=LINT_CHECKS,
            fail_fast=ns.fail_fast,
            report_format=ns.report,
            emit_artifacts=ns.emit_artifacts,
        )

    if ns.ops_cmd == "surface":
        if ns.fix:
            return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_surface_meta.py"], ns.report)
        return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_surface_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-check":
        return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"], ns.report)

    if ns.ops_cmd == "suites-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-orphan-suite.py"],
            ns.report,
        )

    if ns.ops_cmd == "schema-check":
        return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"], ns.report)

    if ns.ops_cmd == "tool-versions-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-floating-tool-versions.py"],
            ns.report,
        )

    if ns.ops_cmd == "no-direct-script-usage-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-direct-script-usage.py"],
            ns.report,
        )

    if ns.ops_cmd == "directory-budgets-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/scripts/check_scripts_submodules.py", "--threshold", "25"],
            ns.report,
        )

    if ns.ops_cmd == "naming-check":
        return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/checks/check_ops_script_names.py"], ns.report)

    if ns.ops_cmd == "layer-drift-check":
        return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/contracts/policies/check_layer_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-index":
        cmd = [*SELF_CLI, "docs", "generate", "--report", "text"]
        return _run_simple_cmd(ctx, cmd, ns.report)
    if ns.ops_cmd == "policy-audit":
        return _ops_policy_audit(ctx, ns.report)
    if ns.ops_cmd == "k8s-flakes-check":
        code, output = _k8s_flakes(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-test-contract":
        code, output = _k8s_test_contract(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-test-lib-contract":
        code, output = _k8s_test_lib(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-checks-layout":
        code, output = _k8s_checks_layout(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-surface-generate":
        code, output = _k8s_surface_generate(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd in {"clean-generated", "clean"}:
        return _ops_clean_generated(ctx, ns.report, ns.force)

    return 2


def configure_ops_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ops", help="ops lint and contracts command surface")
    ops_sub = p.add_subparsers(dest="ops_cmd", required=True)

    check = ops_sub.add_parser("check", help="run canonical ops/check lane")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--fix", action="store_true")

    lint = ops_sub.add_parser("lint", help="run canonical ops lint checks")
    lint.add_argument("--report", choices=["text", "json"], default="text")
    lint.add_argument("--fail-fast", action="store_true")
    lint.add_argument("--emit-artifacts", action="store_true")
    lint.add_argument("--fix", action="store_true")

    for name, help_text in (
        ("surface", "validate or generate ops surface metadata"),
        ("contracts-check", "validate ops contracts index and schema pairs"),
        ("suites-check", "validate ops suite references"),
        ("schema-check", "validate ops schema contracts"),
        ("tool-versions-check", "validate pinned ops tool versions"),
        ("no-direct-script-usage-check", "validate direct ops script usage policy"),
        ("directory-budgets-check", "validate ops-related directory budgets"),
        ("naming-check", "validate ops naming conventions"),
        ("layer-drift-check", "validate cross-layer drift rules"),
        ("contracts-index", "generate ops contracts docs index"),
        ("policy-audit", "validate ops policy configs reflected in ops usage"),
        ("k8s-surface-generate", "generate k8s test surface docs from manifest"),
        ("k8s-checks-layout", "validate k8s checks layout budget"),
        ("k8s-test-lib-contract", "validate k8s tests checks/_lib helper contract"),
        ("k8s-flakes-check", "evaluate k8s flake report and quarantine policy"),
        ("k8s-test-contract", "validate k8s test manifest ownership/contract"),
        ("clean-generated", "remove runtime evidence files under ops/_generated"),
        ("clean", "alias for clean-generated"),
    ):
        cmd = ops_sub.add_parser(name, help=help_text)
        cmd.add_argument("--report", choices=["text", "json"], default="text")
        cmd.add_argument("--fix", action="store_true")
        if name in {"clean-generated", "clean"}:
            cmd.add_argument("--force", action="store_true")
