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


def _load_ops_env_schema(repo_root: Path, schema: str) -> dict[str, object]:
    schema_path = (repo_root / schema).resolve()
    return json.loads(schema_path.read_text(encoding="utf-8"))


def _ops_env_validate_native(repo_root: Path, schema: str) -> tuple[int, str, dict[str, str]]:
    data = _load_ops_env_schema(repo_root, schema)
    variables = data.get("variables", {})
    if not isinstance(variables, dict):
        return 1, "ops env schema missing variables map", {}
    resolved: dict[str, str] = {}
    for name, spec_any in variables.items():
        if not isinstance(name, str) or not isinstance(spec_any, dict):
            continue
        raw = os.environ.get(name)
        if raw is not None and raw != "":
            resolved[name] = raw
            continue
        default = spec_any.get("default")
        resolved[name] = str(default) if isinstance(default, (str, int, float)) else ""
    errors: list[str] = []
    for name, value in resolved.items():
        if not value:
            errors.append(f"{name} resolved empty")
    if errors:
        return 1, "\n".join(errors), resolved
    return 0, "ops env contract check passed", resolved


def _build_unified_ops_pins(repo_root: Path) -> tuple[int, str]:
    pins_dir = repo_root / "configs" / "ops" / "pins"
    out = repo_root / "configs" / "ops" / "pins.json"
    try:
        tools = json.loads((pins_dir / "tools.json").read_text(encoding="utf-8"))
        images = json.loads((pins_dir / "images.json").read_text(encoding="utf-8"))
        helm = json.loads((pins_dir / "helm.json").read_text(encoding="utf-8"))
        datasets = json.loads((pins_dir / "datasets.json").read_text(encoding="utf-8"))
    except Exception as exc:
        return 1, f"failed reading ops pin inputs: {exc}"
    unified = {
        "schema_version": 1,
        "contract_version": "1.0.0",
        "tools": tools.get("tools", {}),
        "images": images.get("images", {}),
        "helm": helm.get("helm", {}),
        "datasets": datasets.get("datasets", {}),
        "policy": {"allow_pin_bypass": False, "relaxation_registry": "configs/policy/pin-relaxations.json"},
    }
    out.write_text(json.dumps(unified, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(repo_root))


def _sync_stack_versions(repo_root: Path) -> tuple[int, str]:
    src = repo_root / "configs" / "ops" / "tool-versions.json"
    out = repo_root / "ops" / "stack" / "versions.json"
    try:
        payload = json.loads(src.read_text(encoding="utf-8"))
    except Exception as exc:
        return 1, f"failed reading tool versions: {exc}"
    versions = payload.get("tools", {}) if isinstance(payload, dict) else {}
    if not isinstance(versions, dict):
        return 1, "invalid tool versions format"
    out.write_text(json.dumps({"schema_version": 1, "tools": versions}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(repo_root))


def _generate_ops_surface_meta(repo_root: Path) -> tuple[int, str]:
    source = repo_root / "configs" / "ops" / "public-surface.json"
    out = repo_root / "ops" / "_meta" / "surface.json"
    try:
        payload = json.loads(source.read_text(encoding="utf-8"))
    except Exception as exc:
        return 1, f"failed reading ops public surface config: {exc}"
    targets = payload.get("make_targets", [])
    if not isinstance(targets, list):
        return 1, "configs/ops/public-surface.json: make_targets must be a list"
    entrypoints = sorted(
        {
            str(item).strip()
            for item in targets
            if isinstance(item, str) and str(item).strip().startswith("ops-")
        }
        | {"ops-help", "ops-layout-lint", "ops-surface", "ops-e2e-validate"}
    )
    out.write_text(json.dumps({"schema_version": 1, "entrypoints": entrypoints}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(repo_root))


def _emit_ops_status(report_format: str, code: int, output: str) -> int:
    if report_format == "json":
        print(
            json.dumps(
                {
                    "schema_version": 1,
                    "tool": "atlasctl",
                    "status": "pass" if code == 0 else "fail",
                    "output": output,
                },
                sort_keys=True,
            )
        )
    else:
        if output:
            print(output)
    return code


def _load_ops_manifest(ctx: RunContext, manifest_path: str) -> dict[str, object]:
    path = (ctx.repo_root / manifest_path).resolve()
    if not path.exists():
        raise FileNotFoundError(f"manifest not found: {manifest_path}")
    suffix = path.suffix.lower()
    raw = path.read_text(encoding="utf-8")
    if suffix == ".json":
        payload = json.loads(raw)
    elif suffix in {".yaml", ".yml"}:
        try:
            import yaml  # type: ignore
        except ModuleNotFoundError as exc:
            raise RuntimeError("yaml manifest requires PyYAML; install it or use .json manifest") from exc
        payload = yaml.safe_load(raw)
    else:
        raise RuntimeError(f"unsupported manifest format `{suffix}`; use .json/.yaml")
    if not isinstance(payload, dict):
        raise RuntimeError("manifest payload must be an object")
    from atlasctl.contracts.schema.validate import validate

    validate("atlasctl.ops.manifest.v1", payload)
    return payload


def _ops_manifest_run(ctx: RunContext, report_format: str, manifest_path: str, fail_fast: bool) -> int:
    try:
        manifest = _load_ops_manifest(ctx, manifest_path)
    except Exception as exc:
        return _emit_ops_status(report_format, 2, f"ops manifest load/validate failed: {exc}")
    steps = manifest.get("steps", [])
    if not isinstance(steps, list):
        return _emit_ops_status(report_format, 2, "ops manifest `steps` must be a list")
    rows: list[dict[str, object]] = []
    failures: list[str] = []
    for item in steps:
        if not isinstance(item, dict):
            continue
        step_id = str(item.get("id", "")).strip() or "<unnamed>"
        cmd = item.get("command", [])
        allow_failure = bool(item.get("allow_failure", False))
        if not isinstance(cmd, list) or not cmd:
            rows.append({"id": step_id, "status": "fail", "exit_code": 2, "error": "invalid command list"})
            failures.append(step_id)
            if fail_fast:
                break
            continue
        result = run_command([str(part) for part in cmd], ctx.repo_root, ctx=ctx)
        code = int(result.code)
        status = "pass" if code == 0 else ("allowed-fail" if allow_failure else "fail")
        rows.append({"id": step_id, "status": status, "exit_code": code, "command": [str(part) for part in cmd]})
        if code != 0 and not allow_failure:
            failures.append(step_id)
            if fail_fast:
                break
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "ops-manifest-run",
        "status": "pass" if not failures else "fail",
        "manifest": manifest_path,
        "run_id": ctx.run_id,
        "steps": rows,
        "failed_steps": failures,
    }
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"ops run manifest={manifest_path} status={payload['status']}")
        for row in rows:
            print(f"- {row['id']}: {row['status']}")
    return 0 if not failures else 1


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
    if not getattr(ns, "ops_cmd", None) and bool(getattr(ns, "list", False)):
        items = sorted(
            {
                "check",
                "lint",
                "surface",
                "contracts-check",
                "suites-check",
                "schema-check",
                "tool-versions-check",
                "no-direct-script-usage-check",
                "directory-budgets-check",
                "naming-check",
                "layer-drift-check",
                "contracts-index",
                "policy-audit",
                "k8s-surface-generate",
                "k8s-checks-layout",
                "k8s-test-lib-contract",
                "k8s-flakes-check",
                "k8s-test-contract",
                "clean-generated",
                "clean",
                "env",
                "pins",
                "gen",
                "stack",
                "k8s",
                "e2e",
                "obs",
                "kind",
                "load",
                "help",
                "up",
                "down",
                "restart",
            }
        )
        if bool(getattr(ns, "json", False)):
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "group": "ops", "items": items}, sort_keys=True))
        else:
            for item in items:
                print(item)
        return 0
    if ns.ops_cmd == "help":
        readme = ctx.repo_root / "ops" / "INDEX.md"
        return _emit_ops_status(ns.report, 0, readme.read_text(encoding="utf-8"))
    if ns.ops_cmd == "run":
        return _ops_manifest_run(
            ctx,
            ns.report,
            manifest_path=ns.manifest,
            fail_fast=bool(getattr(ns, "fail_fast", False)),
        )

    if ns.ops_cmd == "surface":
        surface = ctx.repo_root / "ops" / "_meta" / "surface.json"
        payload = json.loads(surface.read_text(encoding="utf-8"))
        entrypoints = payload.get("entrypoints", [])
        if ns.report == "json":
            return _emit_ops_status(ns.report, 0, json.dumps({"schema_version": 1, "tool": "atlasctl", "entrypoints": entrypoints}, sort_keys=True))
        text = "\n".join(str(item) for item in entrypoints if isinstance(item, str))
        return _emit_ops_status(ns.report, 0, text)

    if ns.ops_cmd == "up":
        return _run_simple_cmd(ctx, ["bash", "ops/run/stack-up.sh", "--profile", os.environ.get("PROFILE", "kind")], ns.report)

    if ns.ops_cmd == "down":
        return _run_simple_cmd(ctx, ["bash", "ops/run/down.sh"], ns.report)

    if ns.ops_cmd == "restart":
        return _run_simple_cmd(ctx, ["bash", "ops/run/k8s-restart.sh"], ns.report)

    if ns.ops_cmd == "env":
        sub = getattr(ns, "ops_env_cmd", "")
        if sub == "validate":
            schema = getattr(ns, "schema", "configs/ops/env.schema.json")
            code, output, _ = _ops_env_validate_native(ctx.repo_root, schema)
            return _emit_ops_status(ns.report, code, output)
        if sub == "print":
            schema = getattr(ns, "schema", "configs/ops/env.schema.json")
            fmt = getattr(ns, "format", "json")
            code, output, resolved = _ops_env_validate_native(ctx.repo_root, schema)
            if code != 0:
                return _emit_ops_status(ns.report, code, output)
            rendered = json.dumps(dict(sorted(resolved.items())), sort_keys=True) if fmt == "json" else "\n".join(
                f"{key}={resolved[key]}" for key in sorted(resolved)
            )
            return _emit_ops_status(ns.report, 0, rendered)
        return 2

    if ns.ops_cmd == "pins":
        sub = getattr(ns, "ops_pins_cmd", "")
        if sub == "check":
            code, output = _build_unified_ops_pins(ctx.repo_root)
            if code != 0:
                return _emit_status(ns.report, code, output)
            steps = [
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/pins/check_ops_pins.py"],
                ["python3", "ops/_lint/pin-relaxations-audit.py"],
                ["bash", "ops/k8s/tests/checks/obs/test_helm_repo_pinning.sh"],
                ["bash", "-lc", "make -s ops-kind-version-drift-test"],
            ]
            for cmd in steps:
                code, output = _run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if sub == "update":
            code, output = _build_unified_ops_pins(ctx.repo_root)
            return _emit_ops_status(ns.report, code, output)
        return 2

    if ns.ops_cmd == "gen":
        sub = getattr(ns, "ops_gen_cmd", "run")
        if sub == "run":
            code, output = _sync_stack_versions(ctx.repo_root)
            if code != 0:
                return _emit_ops_status(ns.report, code, output)
            code, output = _build_unified_ops_pins(ctx.repo_root)
            if code != 0:
                return _emit_ops_status(ns.report, code, output)
            code, output = _generate_ops_surface_meta(ctx.repo_root)
            if code != 0:
                return _emit_ops_status(ns.report, code, output)
            steps = [
                [*SELF_CLI, "docs", "generate", "--report", "text"],
                [*SELF_CLI, "contracts", "generate", "--generators", "chart-schema"],
            ]
            for cmd in steps:
                code, output = _run_check(cmd, ctx.repo_root)
                if output and ns.report != "json":
                    print(output)
                if code != 0:
                    return code
            if ns.report == "json":
                print(
                    json.dumps(
                        {
                            "schema_version": 1,
                            "tool": "atlasctl",
                            "run_id": ctx.run_id,
                            "status": "pass",
                            "action": "ops-gen",
                        },
                        sort_keys=True,
                    )
                )
            return 0
        if sub == "check":
            code = run_ops_command(
                ctx,
                argparse.Namespace(ops_cmd="gen", ops_gen_cmd="run", report=ns.report),
            )
            if code != 0:
                return code
            diff_cmd = [
                "git",
                "diff",
                "--exit-code",
                "--",
                "ops/_generated_committed",
                "docs/_generated/ops-*.md",
                "docs/_generated/layer-contract.md",
                "ops/k8s/charts/bijux-atlas/values.schema.json",
                "ops/stack/versions.json",
            ]
            return _run_simple_cmd(ctx, diff_cmd, ns.report)
        return 2

    if ns.ops_cmd in {"stack", "k8s", "e2e", "obs", "kind", "load"}:
        # Domain tree front-doors: keep shape stable even where implementations are delegated.
        sub_name = {
            "stack": "ops_stack_cmd",
            "k8s": "ops_k8s_cmd",
            "e2e": "ops_e2e_cmd",
            "obs": "ops_obs_cmd",
            "kind": "ops_kind_cmd",
            "load": "ops_load_cmd",
        }[ns.ops_cmd]
        sub = getattr(ns, sub_name, "")
        if ns.ops_cmd == "k8s" and sub == "contracts":
            return _run_simple_cmd(
                ctx,
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"],
                ns.report,
            )
        if ns.ops_cmd == "e2e" and sub == "validate":
            for cmd in (
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/policies/scenarios/check_e2e_suites.py"],
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/policies/scenarios/check_e2e_scenarios.py"],
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/policies/scenarios/check_realdata_scenarios.py"],
            ):
                code, output = _run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "obs" and sub == "verify":
            return _run_simple_cmd(ctx, ["bash", "ops/run/obs-verify.sh"], ns.report)
        if ns.ops_cmd == "obs" and sub == "drill":
            drill = getattr(ns, "drill", "")
            if not drill:
                return _emit_ops_status(ns.report, 2, "missing --drill")
            return _run_simple_cmd(ctx, ["bash", "ops/obs/scripts/bin/run_drill.sh", drill], ns.report)
        if ns.ops_cmd == "stack" and sub == "versions-sync":
            return _run_simple_cmd(
                ctx,
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_stack_versions.py"],
                ns.report,
            )
        if ns.ops_cmd == "stack" and sub == "up":
            profile = getattr(ns, "profile", "kind")
            return _run_simple_cmd(ctx, ["bash", "ops/run/stack-up.sh", "--profile", profile], ns.report)
        if ns.ops_cmd == "stack" and sub == "down":
            return _run_simple_cmd(ctx, ["bash", "ops/run/stack-down.sh"], ns.report)
        if ns.ops_cmd == "stack" and sub == "restart":
            return _run_simple_cmd(ctx, ["bash", "ops/run/k8s-restart.sh"], ns.report)
        if ns.ops_cmd == "kind" and sub == "up":
            return _run_simple_cmd(ctx, ["bash", "ops/stack/kind/up.sh"], ns.report)
        if ns.ops_cmd == "kind" and sub == "down":
            return _run_simple_cmd(ctx, ["bash", "ops/stack/kind/down.sh"], ns.report)
        if ns.ops_cmd == "kind" and sub == "reset":
            return _run_simple_cmd(ctx, ["bash", "ops/stack/kind/reset.sh"], ns.report)
        if ns.ops_cmd == "kind" and sub == "validate":
            steps = [
                ["bash", "ops/stack/kind/context_guard.sh"],
                ["bash", "ops/stack/kind/namespace_guard.sh"],
                ["bash", "ops/k8s/tests/checks/rollout/test_cluster_sanity.sh"],
                ["bash", "ops/k8s/tests/checks/rollout/test_kind_image_resolution.sh"],
                ["bash", "ops/k8s/tests/checks/rollout/test_kind_version_drift.sh"],
                ["bash", "ops/vendor/layout-checks/check_kind_cluster_contract_drift.sh"],
            ]
            for cmd in steps:
                code, output = _run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "kind" and sub == "fault":
            fault = getattr(ns, "name", "")
            if fault == "disk-pressure":
                return _run_simple_cmd(ctx, ["bash", "ops/stack/faults/inject.sh", "fill-node-disk", os.environ.get("MODE", "fill")], ns.report)
            if fault == "latency":
                return _run_simple_cmd(
                    ctx,
                    ["bash", "ops/stack/faults/inject.sh", "toxiproxy-latency", os.environ.get("LATENCY_MS", "250"), os.environ.get("JITTER_MS", "25")],
                    ns.report,
                )
            if fault == "cpu-throttle":
                return _run_simple_cmd(ctx, ["bash", "ops/stack/faults/inject.sh", "cpu-throttle"], ns.report)
            return _emit_ops_status(ns.report, 2, f"unsupported fault `{fault}`")
        if ns.ops_cmd == "e2e" and sub == "run":
            suite = getattr(ns, "suite", "smoke")
            return _run_simple_cmd(ctx, ["bash", "ops/run/e2e.sh", "--suite", suite], ns.report)
        if ns.ops_cmd == "load" and sub == "run":
            suite = getattr(ns, "suite", "mixed-80-20")
            return _run_simple_cmd(ctx, ["env", f"SUITE={suite}", "bash", "ops/run/load-suite.sh"], ns.report)
        return 2

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
        return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/ops/impl/check_ops_script_names.py"], ns.report)

    if ns.ops_cmd == "layer-drift-check":
        return _run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/policies/policies/check_layer_drift.py"], ns.report)

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
    p = sub.add_parser("ops", help="ops control-plane command surface")
    p.add_argument("--list", action="store_true", help="list available ops commands")
    p.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    ops_sub = p.add_subparsers(dest="ops_cmd", required=False)

    check = ops_sub.add_parser("check", help="run canonical ops/check lane")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--fix", action="store_true")
    help_cmd = ops_sub.add_parser("help", help="show canonical ops runbook index")
    help_cmd.add_argument("--report", choices=["text", "json"], default="text")
    up_cmd = ops_sub.add_parser("up", help="bring up full local ops environment")
    up_cmd.add_argument("--report", choices=["text", "json"], default="text")
    down_cmd = ops_sub.add_parser("down", help="tear down full local ops environment")
    down_cmd.add_argument("--report", choices=["text", "json"], default="text")
    restart_cmd = ops_sub.add_parser("restart", help="restart deployed atlas workloads safely")
    restart_cmd.add_argument("--report", choices=["text", "json"], default="text")
    run_cmd = ops_sub.add_parser("run", help="run ops workflow manifest")
    run_cmd.add_argument("--report", choices=["text", "json"], default="text")
    run_cmd.add_argument("--manifest", required=True, help="ops workflow manifest path (.json/.yaml)")
    run_cmd.add_argument("--fail-fast", action="store_true", help="stop on first failing manifest step")

    lint = ops_sub.add_parser("lint", help="run canonical ops lint checks")
    lint.add_argument("--report", choices=["text", "json"], default="text")
    lint.add_argument("--fail-fast", action="store_true")
    lint.add_argument("--emit-artifacts", action="store_true")
    lint.add_argument("--fix", action="store_true")

    env = ops_sub.add_parser("env", help="ops environment commands")
    env.add_argument("--report", choices=["text", "json"], default="text")
    env_sub = env.add_subparsers(dest="ops_env_cmd", required=True)
    env_validate = env_sub.add_parser("validate", help="validate ops env contract")
    env_validate.add_argument("--schema", default="configs/ops/env.schema.json")
    env_print = env_sub.add_parser("print", help="print resolved ops env settings")
    env_print.add_argument("--schema", default="configs/ops/env.schema.json")
    env_print.add_argument("--format", choices=["json", "text"], default="json")

    pins = ops_sub.add_parser("pins", help="ops pins commands")
    pins.add_argument("--report", choices=["text", "json"], default="text")
    pins_sub = pins.add_subparsers(dest="ops_pins_cmd", required=True)
    pins_sub.add_parser("check", help="validate pinned ops versions and drift contracts")
    pins_sub.add_parser("update", help="update ops pins")

    gen = ops_sub.add_parser("gen", help="ops generated artifacts commands")
    gen.add_argument("--report", choices=["text", "json"], default="text")
    gen_sub = gen.add_subparsers(dest="ops_gen_cmd", required=False)
    gen_sub.add_parser("run", help="regenerate committed ops outputs")
    gen_sub.add_parser("check", help="regenerate then fail on drift")

    stack = ops_sub.add_parser("stack", help="ops stack commands")
    stack.add_argument("--report", choices=["text", "json"], default="text")
    stack_sub = stack.add_subparsers(dest="ops_stack_cmd", required=True)
    stack_sub.add_parser("versions-sync", help="sync stack versions json from tool versions SSOT")
    stack_up = stack_sub.add_parser("up", help="bring up stack components")
    stack_up.add_argument("--profile", default="kind")
    stack_sub.add_parser("down", help="tear down stack components")
    stack_sub.add_parser("restart", help="restart atlas deployment")

    k8s = ops_sub.add_parser("k8s", help="ops kubernetes commands")
    k8s.add_argument("--report", choices=["text", "json"], default="text")
    k8s_sub = k8s.add_subparsers(dest="ops_k8s_cmd", required=True)
    k8s_sub.add_parser("contracts", help="validate k8s contracts")

    e2e = ops_sub.add_parser("e2e", help="ops end-to-end commands")
    e2e.add_argument("--report", choices=["text", "json"], default="text")
    e2e_sub = e2e.add_subparsers(dest="ops_e2e_cmd", required=True)
    e2e_sub.add_parser("validate", help="validate e2e scenarios and suites")
    e2e_run = e2e_sub.add_parser("run", help="run e2e suite")
    e2e_run.add_argument("--suite", choices=["smoke", "k8s-suite", "realdata"], default="smoke")

    obs = ops_sub.add_parser("obs", help="ops observability commands")
    obs.add_argument("--report", choices=["text", "json"], default="text")
    obs_sub = obs.add_subparsers(dest="ops_obs_cmd", required=True)
    obs_sub.add_parser("verify", help="run observability verification")
    obs_drill = obs_sub.add_parser("drill", help="run one observability drill")
    obs_drill.add_argument("--drill", required=True)

    kind = ops_sub.add_parser("kind", help="kind substrate commands")
    kind.add_argument("--report", choices=["text", "json"], default="text")
    kind_sub = kind.add_subparsers(dest="ops_kind_cmd", required=True)
    kind_sub.add_parser("up", help="create kind cluster")
    kind_sub.add_parser("down", help="delete kind cluster")
    kind_sub.add_parser("reset", help="reset kind cluster")
    kind_sub.add_parser("validate", help="validate kind substrate contracts")
    kind_fault = kind_sub.add_parser("fault", help="inject kind fault")
    kind_fault.add_argument("name", choices=["disk-pressure", "latency", "cpu-throttle"])

    load = ops_sub.add_parser("load", help="ops load commands")
    load.add_argument("--report", choices=["text", "json"], default="text")
    load_sub = load.add_subparsers(dest="ops_load_cmd", required=True)
    load_run = load_sub.add_parser("run", help="run load suite")
    load_run.add_argument("--suite", default="mixed-80-20")

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
