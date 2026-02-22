from __future__ import annotations

import argparse
import json
import os

from . import ops_runtime_commands as impl

def run_ops_command(ctx, ns: argparse.Namespace) -> int:
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
                "deploy",
                "k8s",
                "e2e",
                "obs",
                "kind",
                "load",
                "datasets",
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
        return impl._emit_ops_status(ns.report, 0, readme.read_text(encoding="utf-8"))
    if ns.ops_cmd == "run":
        return impl._ops_manifest_run(
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
            return impl._emit_ops_status(ns.report, 0, json.dumps({"schema_version": 1, "tool": "atlasctl", "entrypoints": entrypoints}, sort_keys=True))
        text = "\n".join(str(item) for item in entrypoints if isinstance(item, str))
        return impl._emit_ops_status(ns.report, 0, text)

    if ns.ops_cmd == "up":
        return impl._run_simple_cmd(ctx, ["bash", "ops/run/stack-up.sh", "--profile", os.environ.get("PROFILE", "kind")], ns.report)

    if ns.ops_cmd == "down":
        return impl._run_simple_cmd(ctx, ["bash", "ops/run/down.sh"], ns.report)

    if ns.ops_cmd == "restart":
        return impl._run_simple_cmd(ctx, ["bash", "ops/run/k8s-restart.sh"], ns.report)

    if ns.ops_cmd == "deploy":
        return impl._run_simple_cmd(ctx, ["bash", "ops/run/deploy-atlas.sh"], ns.report)

    if ns.ops_cmd == "env":
        sub = getattr(ns, "ops_env_cmd", "")
        if sub == "validate":
            schema = getattr(ns, "schema", "configs/ops/env.schema.json")
            code, output, _ = impl._ops_env_validate_native(ctx.repo_root, schema)
            return impl._emit_ops_status(ns.report, code, output)
        if sub == "print":
            schema = getattr(ns, "schema", "configs/ops/env.schema.json")
            fmt = getattr(ns, "format", "json")
            code, output, resolved = impl._ops_env_validate_native(ctx.repo_root, schema)
            if code != 0:
                return impl._emit_ops_status(ns.report, code, output)
            rendered = json.dumps(dict(sorted(resolved.items())), sort_keys=True) if fmt == "json" else "\n".join(
                f"{key}={resolved[key]}" for key in sorted(resolved)
            )
            return impl._emit_ops_status(ns.report, 0, rendered)
        return 2

    if ns.ops_cmd == "pins":
        sub = getattr(ns, "ops_pins_cmd", "")
        if sub == "check":
            code, output = impl._build_unified_ops_pins(ctx.repo_root)
            if code != 0:
                return impl._emit_status(ns.report, code, output)
            steps = [
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/pins/check_ops_pins.py"],
                ["python3", "ops/_lint/pin-relaxations-audit.py"],
                ["bash", "ops/k8s/tests/checks/obs/contracts/test_helm_repo_pinning.sh"],
                ["bash", "-lc", "make -s ops-kind-version-drift-test"],
            ]
            for cmd in steps:
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if sub == "update":
            code, output = impl._build_unified_ops_pins(ctx.repo_root)
            return impl._emit_ops_status(ns.report, code, output)
        return 2

    if ns.ops_cmd == "gen":
        sub = getattr(ns, "ops_gen_cmd", "run")
        if sub == "run":
            code, output = impl._sync_stack_versions(ctx.repo_root)
            if code != 0:
                return impl._emit_ops_status(ns.report, code, output)
            code, output = impl._build_unified_ops_pins(ctx.repo_root)
            if code != 0:
                return impl._emit_ops_status(ns.report, code, output)
            code, output = impl._generate_ops_surface_meta(ctx.repo_root)
            if code != 0:
                return impl._emit_ops_status(ns.report, code, output)
            steps = [
                [*impl.SELF_CLI, "docs", "generate", "--report", "text"],
                [*impl.SELF_CLI, "contracts", "generate", "--generators", "chart-schema"],
            ]
            for cmd in steps:
                code, output = impl._run_check(cmd, ctx.repo_root)
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
            return impl._run_simple_cmd(ctx, diff_cmd, ns.report)
        return 2

    if ns.ops_cmd in {"stack", "k8s", "e2e", "obs", "kind", "load", "datasets"}:
        # Domain tree front-doors: keep shape stable even where implementations are delegated.
        sub_name = {
            "stack": "ops_stack_cmd",
            "k8s": "ops_k8s_cmd",
            "e2e": "ops_e2e_cmd",
            "obs": "ops_obs_cmd",
            "kind": "ops_kind_cmd",
            "load": "ops_load_cmd",
            "datasets": "ops_datasets_cmd",
        }[ns.ops_cmd]
        sub = getattr(ns, sub_name, "")
        if ns.ops_cmd == "k8s" and sub == "contracts":
            return impl._run_simple_cmd(
                ctx,
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"],
                ns.report,
            )
        if ns.ops_cmd == "e2e" and sub == "validate":
            for cmd in (
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/policies/scenarios/check_e2e_suites.py"],
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/policies/scenarios/check_e2e_scenarios.py"],
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/policies/scenarios/check_realdata_scenarios.py"],
            ):
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "obs" and sub == "verify":
            return impl._run_simple_cmd(ctx, ["bash", "ops/run/obs-verify.sh"], ns.report)
        if ns.ops_cmd == "obs" and sub == "drill":
            drill = getattr(ns, "drill", "")
            if not drill:
                return impl._emit_ops_status(ns.report, 2, "missing --drill")
            return impl._run_simple_cmd(ctx, ["bash", "ops/obs/scripts/run_drill.sh", drill], ns.report)
        if ns.ops_cmd == "stack" and sub == "versions-sync":
            return impl._run_simple_cmd(
                ctx,
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_stack_versions.py"],
                ns.report,
            )
        if ns.ops_cmd == "stack" and sub == "up":
            profile = getattr(ns, "profile", "kind")
            return impl._run_simple_cmd(ctx, ["bash", "ops/run/stack-up.sh", "--profile", profile], ns.report)
        if ns.ops_cmd == "stack" and sub == "down":
            return impl._run_simple_cmd(ctx, ["bash", "ops/run/stack-down.sh"], ns.report)
        if ns.ops_cmd == "stack" and sub == "restart":
            return impl._run_simple_cmd(ctx, ["bash", "ops/run/k8s-restart.sh"], ns.report)
        if ns.ops_cmd == "kind" and sub == "up":
            return impl._run_simple_cmd(ctx, ["bash", "ops/stack/kind/up.sh"], ns.report)
        if ns.ops_cmd == "kind" and sub == "down":
            return impl._run_simple_cmd(ctx, ["bash", "ops/stack/kind/down.sh"], ns.report)
        if ns.ops_cmd == "kind" and sub == "reset":
            return impl._run_simple_cmd(ctx, ["bash", "ops/stack/kind/reset.sh"], ns.report)
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
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "kind" and sub == "fault":
            fault = getattr(ns, "name", "")
            if fault == "disk-pressure":
                return impl._run_simple_cmd(ctx, ["bash", "ops/stack/faults/inject.sh", "fill-node-disk", os.environ.get("MODE", "fill")], ns.report)
            if fault == "latency":
                return impl._run_simple_cmd(
                    ctx,
                    ["bash", "ops/stack/faults/inject.sh", "toxiproxy-latency", os.environ.get("LATENCY_MS", "250"), os.environ.get("JITTER_MS", "25")],
                    ns.report,
                )
            if fault == "cpu-throttle":
                return impl._run_simple_cmd(ctx, ["bash", "ops/stack/faults/inject.sh", "cpu-throttle"], ns.report)
            return impl._emit_ops_status(ns.report, 2, f"unsupported fault `{fault}`")
        if ns.ops_cmd == "e2e" and sub == "run":
            suite = getattr(ns, "suite", "smoke")
            return impl._run_simple_cmd(ctx, ["bash", "ops/run/e2e.sh", "--suite", suite], ns.report)
        if ns.ops_cmd == "load" and sub == "run":
            suite = getattr(ns, "suite", "mixed-80-20")
            return impl._run_simple_cmd(ctx, ["env", f"SUITE={suite}", "bash", "ops/run/load-suite.sh"], ns.report)
        if ns.ops_cmd == "datasets" and sub == "verify":
            return impl._run_simple_cmd(ctx, ["bash", "ops/run/datasets-verify.sh"], ns.report)
        if ns.ops_cmd == "datasets" and sub == "fetch":
            return impl._run_simple_cmd(ctx, ["bash", "ops/run/warm.sh"], ns.report)
        if ns.ops_cmd == "datasets" and sub == "pin":
            return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/datasets/build_manifest_lock.py"], ns.report)
        return 2

    if ns.ops_cmd == "check":
        steps = [
            [*impl.SELF_CLI, "ops", "lint", "--report", ns.report, "--emit-artifacts"],
            [*impl.SELF_CLI, "ops", "contracts-check", "--report", ns.report],
            [*impl.SELF_CLI, "ops", "suites-check", "--report", ns.report],
            [*impl.SELF_CLI, "ops", "schema-check", "--report", ns.report],
            ["env", "CACHE_STATUS_STRICT=0", "make", "-s", "ops-cache-status"],
            ["make", "-s", "pins/check"],
            [*impl.SELF_CLI, "ops", "surface", "--report", ns.report],
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_index_surface.py"],
        ]
        for cmd in steps:
            code, output = impl._run_check(cmd, ctx.repo_root)
            if output:
                print(output)
            if code != 0:
                return code
        return 0

    if ns.ops_cmd == "lint":
        if ns.fix:
            for cmd in (
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_surface_meta.py"],
                [*impl.SELF_CLI, "docs", "generate", "--report", "text"],
            ):
                code, output = impl._run_check(cmd, ctx.repo_root)
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
            return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_surface_meta.py"], ns.report)
        return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/surface/check_ops_surface_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-check":
        return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"], ns.report)

    if ns.ops_cmd == "suites-check":
        return impl._run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-orphan-suite.py"],
            ns.report,
        )

    if ns.ops_cmd == "schema-check":
        return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"], ns.report)

    if ns.ops_cmd == "tool-versions-check":
        return impl._run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-floating-tool-versions.py"],
            ns.report,
        )

    if ns.ops_cmd == "no-direct-script-usage-check":
        return impl._run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-direct-script-usage.py"],
            ns.report,
        )

    if ns.ops_cmd == "directory-budgets-check":
        return impl._run_simple_cmd(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/scripts/check_scripts_submodules.py", "--threshold", "25"],
            ns.report,
        )

    if ns.ops_cmd == "naming-check":
        return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/domains/ops/ops_checks/impl/check_ops_script_names.py"], ns.report)

    if ns.ops_cmd == "layer-drift-check":
        return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/policies/policies/check_layer_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-index":
        cmd = [*impl.SELF_CLI, "docs", "generate", "--report", "text"]
        return impl._run_simple_cmd(ctx, cmd, ns.report)
    if ns.ops_cmd == "policy-audit":
        return _ops_policy_audit(ctx, ns.report)
    if ns.ops_cmd == "k8s-flakes-check":
        code, output = impl._k8s_flakes(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-test-contract":
        code, output = impl._k8s_test_contract(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-test-lib-contract":
        code, output = impl._k8s_test_lib(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-checks-layout":
        code, output = impl._k8s_checks_layout(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd == "k8s-surface-generate":
        code, output = impl._k8s_surface_generate(ctx.repo_root)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code
    if ns.ops_cmd in {"clean-generated", "clean"}:
        return impl._ops_clean_generated(ctx, ns.report, ns.force)

    return 2

