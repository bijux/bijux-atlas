from __future__ import annotations

import argparse
import json
import os
from pathlib import Path

from . import ops_runtime_commands as impl
from atlasctl.core.process import shell_script_command

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
                "cache",
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
        manifest_path = getattr(ns, "manifest", None)
        task_name = getattr(ns, "task", None)
        if manifest_path is None and task_name:
            try:
                manifest_path = impl._ops_task_manifest(ctx, str(task_name))
            except Exception as exc:
                return impl._emit_ops_status(getattr(ns, "report", "text"), 2, str(exc))
        if not manifest_path:
            return impl._emit_ops_status(getattr(ns, "report", "text"), 2, "provide <task> or --manifest")
        return impl._ops_manifest_run(
            ctx,
            ns.report,
            manifest_path=manifest_path,
            fail_fast=bool(getattr(ns, "fail_fast", False)),
        )
    if ns.ops_cmd == "run-script":
        script = str(getattr(ns, "script", "")).strip()
        if not script:
            return impl._emit_ops_status(getattr(ns, "report", "text"), 2, "missing ops/run script path")
        path = Path("ops/run") / script
        if path.is_absolute() or ".." in path.parts:
            return impl._emit_ops_status(getattr(ns, "report", "text"), 2, "invalid script path; use ops/run relative path")
        print("deprecated: `atlasctl ops run-script` is a temporary migration shim; use explicit `atlasctl ops/...` commands")
        return impl._run_simple_cmd(ctx, shell_script_command(path.as_posix(), *list(getattr(ns, "args", []))), getattr(ns, "report", "text"))
    if ns.ops_cmd == "list":
        if getattr(ns, "kind", "") == "tasks":
            return impl._ops_list_tasks(ctx, ns.report)
        return 2
    if ns.ops_cmd == "explain":
        return impl._ops_explain_task(ctx, ns.report, getattr(ns, "task", ""))

    if ns.ops_cmd == "surface":
        surface = ctx.repo_root / "ops" / "_meta" / "surface.json"
        payload = json.loads(surface.read_text(encoding="utf-8"))
        entrypoints = payload.get("entrypoints", [])
        if ns.report == "json":
            return impl._emit_ops_status(ns.report, 0, json.dumps({"schema_version": 1, "tool": "atlasctl", "entrypoints": entrypoints}, sort_keys=True))
        text = "\n".join(str(item) for item in entrypoints if isinstance(item, str))
        return impl._emit_ops_status(ns.report, 0, text)

    if ns.ops_cmd == "up":
        return impl._run_simple_cmd(ctx, shell_script_command("ops/run/stack-up.sh", "--profile", os.environ.get("PROFILE", "kind")), ns.report)

    if ns.ops_cmd == "down":
        return impl._run_simple_cmd(ctx, shell_script_command("ops/run/down.sh"), ns.report)

    if ns.ops_cmd == "restart":
        return impl._run_simple_cmd(ctx, shell_script_command("ops/run/k8s-restart.sh"), ns.report)

    if ns.ops_cmd == "deploy":
        sub = str(getattr(ns, "ops_deploy_cmd", "") or "").strip()
        if sub in {"", "apply"}:
            allow_apply = bool(os.environ.get("CI")) or str(os.environ.get("ATLASCTL_OPS_DEPLOY_ALLOW_APPLY", "")).strip().lower() in {"1", "true", "yes", "on"}
            if not allow_apply:
                return impl._emit_ops_status(ns.report, 2, "deploy apply is gated; set ATLASCTL_OPS_DEPLOY_ALLOW_APPLY=1 or run in CI")
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/deploy-atlas.sh"), ns.report)
        if sub == "plan":
            payload = {
                "schema_version": 1,
                "kind": "ops-deploy-plan",
                "run_id": ctx.run_id,
                "status": "ok",
                "steps": [
                    "validate ops env",
                    "validate stack/kind substrate",
                    "render/apply chart manifests",
                    "rollout health checks",
                ],
                "gates": {
                    "ci_or_explicit_local_gate_required": True,
                    "env_var": "ATLASCTL_OPS_DEPLOY_ALLOW_APPLY",
                },
            }
            if ns.report == "json":
                print(json.dumps(payload, sort_keys=True))
            else:
                print("ops deploy plan")
                for step in payload["steps"]:
                    print(f"- {step}")
            return 0
        if sub == "rollback":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/undeploy.sh"), ns.report)
        return 2

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

    if ns.ops_cmd in {"stack", "k8s", "e2e", "obs", "kind", "load", "cache", "datasets"}:
        # Domain tree front-doors: keep shape stable even where implementations are delegated.
        sub_name = {
            "stack": "ops_stack_cmd",
            "k8s": "ops_k8s_cmd",
            "e2e": "ops_e2e_cmd",
            "obs": "ops_obs_cmd",
            "kind": "ops_kind_cmd",
            "load": "ops_load_cmd",
            "cache": "ops_cache_cmd",
            "datasets": "ops_datasets_cmd",
        }[ns.ops_cmd]
        sub = getattr(ns, sub_name, "")
        if ns.ops_cmd == "k8s" and sub == "contracts":
            return impl._run_simple_cmd(
                ctx,
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"],
                ns.report,
            )
        if ns.ops_cmd == "k8s" and sub == "check":
            for cmd in (
                [*impl.SELF_CLI, "ops", "k8s", "--report", ns.report, "contracts"],
                [*impl.SELF_CLI, "ops", "kind", "--report", ns.report, "validate"],
            ):
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "k8s" and sub == "render":
            env_name = str(getattr(ns, "env", "kind") or "kind")
            payload = impl._ops_k8s_render_summary(ctx.repo_root, env_name=env_name, run_id=ctx.run_id)
            out_rel = str(getattr(ns, "out", "") or "artifacts/reports/atlasctl/ops-k8s-render.json")
            written = impl._write_json_report(ctx.repo_root, out_rel, payload)
            if ns.report == "json":
                print(json.dumps({"schema_version": 1, "status": "ok", "kind": payload["kind"], "out": written, "render_hash": payload["render_hash"]}, sort_keys=True))
            else:
                print(written)
            return 0
        if ns.ops_cmd == "k8s" and sub == "validate":
            in_rel = str(getattr(ns, "in_file", "") or "artifacts/reports/atlasctl/ops-k8s-render.json")
            path = ctx.repo_root / in_rel
            if not path.exists():
                return impl._emit_ops_status(ns.report, 2, f"missing k8s render summary: {in_rel}")
            payload = json.loads(path.read_text(encoding="utf-8"))
            errors = impl._validate_ops_k8s_render_payload(payload)
            return impl._emit_ops_status(ns.report, 0 if not errors else 1, "k8s render summary valid" if not errors else "\n".join(errors))
        if ns.ops_cmd == "k8s" and sub == "diff":
            in_rel = str(getattr(ns, "in_file", "") or "artifacts/reports/atlasctl/ops-k8s-render.json")
            golden_rel = str(getattr(ns, "golden", "") or "ops/k8s/tests/goldens/render-kind.summary.json")
            in_path = ctx.repo_root / in_rel
            golden_path = ctx.repo_root / golden_rel
            if not in_path.exists():
                return impl._emit_ops_status(ns.report, 2, f"missing input render summary: {in_rel}")
            if not golden_path.exists():
                return impl._emit_ops_status(ns.report, 2, f"missing golden render summary: {golden_rel}")
            current = json.loads(in_path.read_text(encoding="utf-8"))
            golden = json.loads(golden_path.read_text(encoding="utf-8"))
            current_cmp = {k: v for k, v in current.items() if k != "run_id"}
            golden_cmp = {k: v for k, v in golden.items() if k != "run_id"}
            diff = {
                "schema_version": 1,
                "kind": "ops-k8s-render-diff",
                "status": "pass" if current_cmp == golden_cmp else "fail",
                "in_file": in_rel,
                "golden": golden_rel,
                "current_hash": current.get("render_hash", ""),
                "golden_hash": golden.get("render_hash", ""),
                "current_count": current.get("test_count", 0),
                "golden_count": golden.get("test_count", 0),
            }
            if ns.report == "json":
                print(json.dumps(diff, sort_keys=True))
            else:
                print(f"{diff['status']}: current={diff['current_hash']} golden={diff['golden_hash']}")
            return 0 if diff["status"] == "pass" else 1
        if ns.ops_cmd == "e2e" and sub == "validate":
            for cmd in (
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/scenarios/check_e2e_suites.py"],
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/scenarios/check_e2e_scenarios.py"],
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/scenarios/check_realdata_scenarios.py"],
            ):
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "e2e" and sub == "validate-results":
            in_rel = str(getattr(ns, "in_file", "")).strip()
            if not in_rel:
                return impl._emit_ops_status(ns.report, 2, "missing --in-file")
            path = ctx.repo_root / in_rel
            if not path.exists():
                return impl._emit_ops_status(ns.report, 2, f"missing e2e results file: {in_rel}")
            try:
                payload = json.loads(path.read_text(encoding="utf-8"))
            except Exception as exc:
                return impl._emit_ops_status(ns.report, 1, f"invalid json: {exc}")
            errors: list[str] = []
            if not isinstance(payload, dict):
                errors.append("payload must be object")
            else:
                for key in ("schema_version", "status"):
                    if key not in payload:
                        errors.append(f"missing key `{key}`")
            return impl._emit_ops_status(ns.report, 0 if not errors else 1, "e2e results valid" if not errors else "\n".join(errors))
        if ns.ops_cmd == "obs" and sub == "verify":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/obs-verify.sh"), ns.report)
        if ns.ops_cmd == "obs" and sub == "check":
            return impl._run_simple_cmd(ctx, [*impl.SELF_CLI, "ops", "obs", "--report", ns.report, "verify"], ns.report)
        if ns.ops_cmd == "obs" and sub == "lint":
            steps = [
                ["python3", "ops/obs/scripts/contracts/check_profile_goldens.py"],
                ["python3", "ops/obs/scripts/contracts/check_metrics_golden.py"],
                ["python3", "ops/obs/scripts/contracts/check_trace_golden.py"],
            ]
            for cmd in steps:
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "obs" and sub == "report":
            payload = {
                "schema_version": 1,
                "kind": "ops-obs-report",
                "run_id": ctx.run_id,
                "status": "ok",
                "artifacts": {
                    "dashboard_golden": "ops/obs/grafana/atlas-observability-dashboard.golden.json",
                    "metrics_golden": "ops/obs/contract/metrics.golden.prom",
                    "trace_golden": "ops/obs/contract/trace-structure.golden.json",
                },
            }
            out_rel = str(getattr(ns, "out", "") or "artifacts/reports/atlasctl/ops-obs-report.json")
            written = impl._write_json_report(ctx.repo_root, out_rel, payload)
            if ns.report == "json":
                print(json.dumps({"schema_version": 1, "kind": "ops-obs-report", "status": "ok", "out": written}, sort_keys=True))
            else:
                print(written)
            return 0
        if ns.ops_cmd == "obs" and sub == "drill":
            drill = getattr(ns, "drill", "")
            if not drill:
                return impl._emit_ops_status(ns.report, 2, "missing --drill")
            return impl._run_simple_cmd(ctx, shell_script_command("ops/obs/scripts/run_drill.sh", drill), ns.report)
        if ns.ops_cmd == "stack" and sub == "versions-sync":
            return impl._run_simple_cmd(
                ctx,
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_stack_versions.py"],
                ns.report,
            )
        if ns.ops_cmd == "stack" and sub == "up":
            profile = getattr(ns, "profile", "kind")
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/stack-up.sh", "--profile", profile), ns.report)
        if ns.ops_cmd == "stack" and sub == "down":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/stack-down.sh"), ns.report)
        if ns.ops_cmd == "stack" and sub == "restart":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/k8s-restart.sh"), ns.report)
        if ns.ops_cmd == "stack" and sub == "check":
            for cmd in (
                [*impl.SELF_CLI, "ops", "stack", "--report", ns.report, "versions-sync"],
                [*impl.SELF_CLI, "ops", "kind", "--report", ns.report, "validate"],
            ):
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "stack" and sub == "status":
            report = impl._ops_stack_contract_report(ctx.repo_root, ctx.run_id)
            if ns.report == "json":
                print(json.dumps(report, sort_keys=True))
            else:
                print(f"stack status: {report['status']} pinned_tools={report['pinned_tool_count']} stack_tools={report['stack_tool_count']}")
            return 0 if report["status"] == "pass" else 1
        if ns.ops_cmd == "stack" and sub == "validate":
            report = impl._ops_stack_contract_report(ctx.repo_root, ctx.run_id)
            errs: list[str] = []
            if report["missing_in_stack_versions"]:
                errs.append(f"missing tools in ops/stack/versions.json: {', '.join(report['missing_in_stack_versions'])}")
            if not report["version_manifest_images"]:
                errs.append("ops/stack/version-manifest.json must contain at least one image")
            return impl._emit_ops_status(ns.report, 0 if not errs else 1, "stack contracts valid" if not errs else "\n".join(errs))
        if ns.ops_cmd == "stack" and sub == "report":
            payload = impl._ops_stack_contract_report(ctx.repo_root, ctx.run_id)
            out_rel = str(getattr(ns, "out", "") or "artifacts/reports/atlasctl/ops-stack-report.json")
            written = impl._write_json_report(ctx.repo_root, out_rel, payload)
            if ns.report == "json":
                print(json.dumps({"schema_version": 1, "kind": "ops-stack-report", "status": payload["status"], "out": written}, sort_keys=True))
            else:
                print(written)
            return 0 if payload["status"] == "pass" else 1
        if ns.ops_cmd == "kind" and sub == "up":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/stack/kind/up.sh"), ns.report)
        if ns.ops_cmd == "kind" and sub == "down":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/stack/kind/down.sh"), ns.report)
        if ns.ops_cmd == "kind" and sub == "reset":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/stack/kind/reset.sh"), ns.report)
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
                return impl._run_simple_cmd(ctx, shell_script_command("ops/stack/faults/inject.sh", "fill-node-disk", os.environ.get("MODE", "fill")), ns.report)
            if fault == "latency":
                return impl._run_simple_cmd(
                    ctx,
                    ["bash", "ops/stack/faults/inject.sh", "toxiproxy-latency", os.environ.get("LATENCY_MS", "250"), os.environ.get("JITTER_MS", "25")],
                    ns.report,
                )
            if fault == "cpu-throttle":
                return impl._run_simple_cmd(ctx, shell_script_command("ops/stack/faults/inject.sh", "cpu-throttle"), ns.report)
            return impl._emit_ops_status(ns.report, 2, f"unsupported fault `{fault}`")
        if ns.ops_cmd == "e2e" and sub == "run":
            suite = getattr(ns, "suite", "smoke")
            scenario = str(getattr(ns, "scenario", "") or "").strip()
            if scenario:
                suite = scenario
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/e2e.sh", "--suite", suite), ns.report)
        if ns.ops_cmd == "load" and sub == "run":
            suite = getattr(ns, "suite", "mixed-80-20")
            return impl._run_simple_cmd(ctx, ["env", f"SUITE={suite}", *shell_script_command("ops/run/load-suite.sh")], ns.report)
        if ns.ops_cmd == "load" and sub == "check":
            return impl._run_simple_cmd(ctx, [*impl.SELF_CLI, "load", "smoke"], ns.report)
        if ns.ops_cmd == "load" and sub == "compare":
            baseline_rel = str(getattr(ns, "baseline", "")).strip()
            current_rel = str(getattr(ns, "current", "")).strip()
            out_rel = str(getattr(ns, "out", "") or "artifacts/reports/atlasctl/ops-load-compare.json")
            if not baseline_rel or not current_rel:
                return impl._emit_ops_status(ns.report, 2, "load compare requires --baseline and --current")
            baseline = json.loads((ctx.repo_root / baseline_rel).read_text(encoding="utf-8"))
            current = json.loads((ctx.repo_root / current_rel).read_text(encoding="utf-8"))
            payload = {
                "schema_version": 1,
                "kind": "ops-load-compare",
                "run_id": ctx.run_id,
                "status": "ok",
                "baseline": baseline_rel,
                "current": current_rel,
                "baseline_keys": sorted(baseline.keys()) if isinstance(baseline, dict) else [],
                "current_keys": sorted(current.keys()) if isinstance(current, dict) else [],
            }
            written = impl._write_json_report(ctx.repo_root, out_rel, payload)
            if ns.report == "json":
                print(json.dumps({"schema_version": 1, "kind": "ops-load-compare", "status": "ok", "out": written}, sort_keys=True))
            else:
                print(written)
            return 0
        if ns.ops_cmd == "cache":
            csub = str(getattr(ns, "ops_cache_cmd", "")).strip()
            if csub == "status":
                return impl._ops_cache_status(ctx, ns.report, bool(getattr(ns, "strict", False)), bool(getattr(ns, "plan", False)))
            if csub == "prune":
                return impl._ops_cache_prune(ctx, ns.report)
            return 2
        if ns.ops_cmd == "datasets" and sub == "verify":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/datasets-verify.sh"), ns.report)
        if ns.ops_cmd == "datasets" and sub == "fetch":
            return impl._run_simple_cmd(ctx, shell_script_command("ops/run/warm.sh"), ns.report)
        if ns.ops_cmd == "datasets" and sub in {"pin", "lock"}:
            return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/build_manifest_lock.py"], ns.report)
        if ns.ops_cmd == "datasets" and sub == "qc":
            qsub = str(getattr(ns, "ops_datasets_qc_cmd", "")).strip()
            args = list(getattr(ns, "args", []))
            if qsub == "summary":
                return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/qc_summary.py", *args], ns.report)
            if qsub == "diff":
                return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/qc_diff.py", *args], ns.report)
            return 2
        if ns.ops_cmd == "datasets" and sub == "validate":
            steps = [
                ["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/catalog_validate.py"],
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/scenarios/check_e2e_scenarios.py"],
            ]
            for cmd in steps:
                code, output = impl._run_check(cmd, ctx.repo_root)
                if output:
                    print(output)
                if code != 0:
                    return code
            return 0
        if ns.ops_cmd == "datasets" and sub == "lint-ids":
            return impl._run_simple_cmd(
                ctx,
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/scripts/dataset_id_lint.py"],
                ns.report,
            )
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
        if bool(getattr(ns, "all", False)):
            steps.extend(
                [
                    [*impl.SELF_CLI, "ops", "k8s", "--report", ns.report, "contracts"],
                    [*impl.SELF_CLI, "ops", "kind", "--report", ns.report, "validate"],
                    [*impl.SELF_CLI, "ops", "e2e", "--report", ns.report, "run", "--suite", "realdata"],
                ]
            )
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
        if bool(getattr(ns, "all", False)):
            return _run_checks(
                ctx,
                checks=LINT_CHECKS,
                fail_fast=ns.fail_fast,
                report_format=ns.report,
                emit_artifacts=True,
            )
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
        return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/domains/ops/ops_checks/check_ops_script_names.py"], ns.report)

    if ns.ops_cmd == "layer-drift-check":
        return impl._run_simple_cmd(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/domains/governance/check_layer_drift.py"], ns.report)

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
