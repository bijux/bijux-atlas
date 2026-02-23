from __future__ import annotations

import json
import os
import re
import shutil
import hashlib
import datetime as dt
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.registry.readers import load_ops_tasks_catalog as load_ops_tasks_catalog_file
from atlasctl.core.process import run_command
from atlasctl.core.runtime.paths import write_text_file
from atlasctl.commands.ops._shared.json_emit import write_ops_json_report
from atlasctl.commands.ops.tools import environment_summary
from atlasctl.commands.ops.runtime_modules.layer_contract import (
    load_layer_contract,
    ns_e2e as lc_ns_e2e,
    ns_k8s as lc_ns_k8s,
    release_default as lc_release_default,
    service_atlas as lc_service_atlas,
)
from atlasctl.commands.ops.k8s.runtime_bridge import (
    _k8s_checks_layout,
    _k8s_flakes,
    _k8s_surface_generate,
    _k8s_test_contract,
    _k8s_test_lib,
)

SELF_CLI = ["./bin/atlasctl"]


def _write_json_report(repo_root: Path, out_rel: str, payload: dict[str, object]) -> str:
    out = Path(out_rel)
    if not out.is_absolute():
        out = repo_root / out_rel
    out.parent.mkdir(parents=True, exist_ok=True)
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    try:
        return out.relative_to(repo_root).as_posix()
    except ValueError:
        return out.as_posix()


def _write_json_report_ctx(ctx: RunContext, area: str, payload: dict[str, object]) -> str:
    out = write_ops_json_report(ctx, area, "report.json", payload)
    try:
        return out.relative_to(ctx.repo_root).as_posix()
    except ValueError:
        return out.as_posix()


def _run_check(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    result = run_command(cmd, repo_root)
    return result.code, result.combined_output


def _run_simple_cmd(ctx: RunContext, cmd: list[str], report_format: str) -> int:
    result = run_command(cmd, ctx.repo_root, ctx=ctx)
    output = result.combined_output.strip()
    return _emit_ops_status(report_format, result.code, output)


def _ops_k8s_render_summary(repo_root: Path, env_name: str, run_id: str) -> dict[str, object]:
    manifest_path = repo_root / "ops" / "k8s" / "tests" / "manifest.json"
    payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    tests = payload.get("tests", []) if isinstance(payload, dict) else []
    rows: list[dict[str, object]] = []
    for row in tests:
        if not isinstance(row, dict):
            continue
        rows.append(
            {
                "script": str(row.get("script", "")).strip(),
                "groups": sorted(str(x) for x in row.get("groups", []) if str(x).strip()),
                "owner": str(row.get("owner", "")).strip(),
                "timeout_seconds": int(row.get("timeout_seconds", 0) or 0),
            }
        )
    rows = sorted(rows, key=lambda r: str(r["script"]))
    digest = hashlib.sha256(json.dumps(rows, sort_keys=True).encode("utf-8")).hexdigest()
    return {
        "schema_version": 1,
        "kind": "ops-k8s-render-summary",
        "run_id": run_id,
        "env": env_name,
        "source_manifest": "ops/k8s/tests/manifest.json",
        "test_count": len(rows),
        "tests": rows,
        "render_hash": digest,
    }


def _validate_ops_k8s_render_payload(payload: dict[str, object]) -> list[str]:
    errors: list[str] = []
    if int(payload.get("schema_version", 0) or 0) != 1:
        errors.append("schema_version must be 1")
    if str(payload.get("kind", "")) != "ops-k8s-render-summary":
        errors.append("kind must be ops-k8s-render-summary")
    if not str(payload.get("env", "")).strip():
        errors.append("env is required")
    tests = payload.get("tests", [])
    if not isinstance(tests, list):
        errors.append("tests must be a list")
        return errors
    prev = ""
    for idx, row in enumerate(tests, start=1):
        if not isinstance(row, dict):
            errors.append(f"tests[{idx}] must be object")
            continue
        script = str(row.get("script", "")).strip()
        if not script:
            errors.append(f"tests[{idx}].script missing")
        if prev and script and script < prev:
            errors.append("tests must be sorted by script for deterministic render output")
        prev = script or prev
        groups = row.get("groups", [])
        if not isinstance(groups, list):
            errors.append(f"tests[{idx}].groups must be list")
    return errors


def _ops_stack_contract_report(repo_root: Path, run_id: str) -> dict[str, object]:
    tools = json.loads((repo_root / "configs/ops/pins/tools.json").read_text(encoding="utf-8"))
    stack_versions = json.loads((repo_root / "ops/stack/versions.json").read_text(encoding="utf-8"))
    version_manifest = json.loads((repo_root / "ops/stack/version-manifest.json").read_text(encoding="utf-8"))
    pinned_tools = tools.get("tools", {}) if isinstance(tools, dict) else {}
    stack_tools = stack_versions.get("tools", {}) if isinstance(stack_versions, dict) else {}
    pinned_names = sorted(k for k in pinned_tools.keys()) if isinstance(pinned_tools, dict) else []
    stack_names = sorted(k for k in stack_tools.keys()) if isinstance(stack_tools, dict) else []
    missing_in_stack = sorted(set(pinned_names) - set(stack_names))
    extra_in_stack = sorted(set(stack_names) - set(pinned_names))
    images = sorted(version_manifest.keys()) if isinstance(version_manifest, dict) else []
    status = "pass" if not missing_in_stack else "fail"
    return {
        "schema_version": 1,
        "kind": "ops-stack-report",
        "run_id": run_id,
        "generated_at_utc": dt.datetime.utcnow().isoformat(timespec="seconds") + "Z",
        "status": status,
        "pins_tools_file": "configs/ops/pins/tools.json",
        "stack_versions_file": "ops/stack/versions.json",
        "version_manifest_file": "ops/stack/version-manifest.json",
        "pinned_tool_count": len(pinned_names),
        "stack_tool_count": len(stack_names),
        "missing_in_stack_versions": missing_in_stack,
        "extra_in_stack_versions": extra_in_stack,
        "version_manifest_images": images,
    }

def load_ops_task_catalog(repo_root: Path) -> dict[str, dict[str, str]]:
    return load_ops_tasks_catalog_file(repo_root)

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
    write_text_file(out, json.dumps(unified, indent=2, sort_keys=True) + "\n", encoding="utf-8")
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
    write_text_file(out, json.dumps({"schema_version": 1, "tools": versions}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
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
    write_text_file(out, json.dumps({"schema_version": 1, "entrypoints": entrypoints}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
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


def _ops_task_manifest(ctx: RunContext, task: str) -> str:
    catalog = load_ops_task_catalog(ctx.repo_root)
    row = catalog.get(task)
    if row is None:
        raise RuntimeError(f"unknown ops task `{task}`")
    manifest = row.get("manifest", "")
    if not manifest:
        raise RuntimeError(f"ops task `{task}` has no manifest")
    return manifest


def _ops_list_tasks(ctx: RunContext, report_format: str) -> int:
    catalog = load_ops_task_catalog(ctx.repo_root)
    items = [
        {"task": name, **meta}
        for name, meta in sorted(catalog.items())
    ]
    if report_format == "json":
        print(json.dumps({"schema_version": 1, "tool": "atlasctl", "kind": "ops-tasks", "tasks": items}, sort_keys=True))
        return 0
    for row in items:
        print(f"{row['task']}: {row['description']} (owner={row['owner']})")
    return 0


def _ops_explain_task(ctx: RunContext, report_format: str, task: str) -> int:
    catalog = load_ops_task_catalog(ctx.repo_root)
    row = catalog.get(task)
    if row is None:
        return _emit_ops_status(report_format, 2, f"unknown ops task `{task}`")
    payload = {"schema_version": 1, "tool": "atlasctl", "task": task, **row}
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
        return 0
    print(f"task: {task}")
    print(f"description: {row.get('description', '')}")
    print(f"manifest: {row.get('manifest', '')}")
    print(f"owner: {row.get('owner', '')}")
    print(f"docs: {row.get('docs', '')}")
    return 0


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


def _ops_cache_status(ctx: RunContext, report_format: str, strict: bool, plan: bool) -> int:
    repo = ctx.repo_root
    cmds: list[list[str]] = [
        ["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_status.py"],
        ["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_budget_check.py"],
    ]
    if strict:
        cmds.append(["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/cache_threshold_check.py"])
    outputs: list[str] = []
    for cmd in cmds:
        result = run_command(cmd, repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
        if result.code != 0:
            return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())
    if plan:
        manifest = json.loads((repo / "ops/datasets/manifest.json").read_text(encoding="utf-8"))
        missing: list[str] = []
        present: list[str] = []
        for ds in manifest.get("datasets", []):
            if not isinstance(ds, dict):
                continue
            name = str(ds.get("name", ""))
            dsid = str(ds.get("id", ""))
            parts = dsid.split("/")
            if len(parts) != 3:
                continue
            release, species, assembly = parts
            store_dir = repo / "artifacts/e2e-store" / f"release={release}" / f"species={species}" / f"assembly={assembly}"
            (present if store_dir.exists() else missing).append(name)
        outputs.append("cache_plan_present=" + ",".join(present))
        outputs.append("cache_plan_would_fetch=" + ",".join(missing))
    return _emit_ops_status(report_format, 0, "\n".join(x for x in outputs if x).strip())


def _ops_cache_prune(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    targets = [
        repo / "artifacts/e2e-store",
        repo / "artifacts/e2e-datasets",
        repo / "artifacts/ops/cache-status",
    ]
    removed: list[str] = []
    for target in targets:
        if target.exists():
            shutil.rmtree(target, ignore_errors=True)
            removed.append(str(target.relative_to(repo)))
    msg = "cache prune completed: " + (" ".join(removed) if removed else "nothing to remove")
    return _emit_ops_status(report_format, 0, msg)


def _ops_prereqs_native(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []
    required = ["docker", "kind", "kubectl", "helm", "k6", "python3"]
    for cmd in required:
        if shutil.which(cmd) is None:
            return _emit_ops_status(report_format, 13, f"missing required tool: {cmd}")
    checks = [
        ["python3", "packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py", "kind", "kubectl", "helm", "k6"],
        ["python3", "packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py", "kind", "kubectl", "helm", "k6", "jq", "yq", "python3"],
        ["python3", "packages/atlasctl/src/atlasctl/layout_checks/check_ops_pins.py"],
        ["python3", "--version"],
        ["kubectl", "version", "--client"],
        ["helm", "version", "--short"],
        ["kind", "version"],
        ["k6", "version"],
    ]
    for cmd in checks:
        result = run_command(cmd, repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
        if result.code != 0:
            return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())
    return _emit_ops_status(report_format, 0, "\n".join(outputs).strip())


def _ops_doctor_native(ctx: RunContext, report_format: str, *, capabilities_json: bool = False) -> int:
    repo = ctx.repo_root
    if capabilities_json:
        payload = json.loads((repo / "configs/ops/command-capabilities.json").read_text(encoding="utf-8"))
        if report_format == "json":
            print(json.dumps(payload, sort_keys=True))
            return 0
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    outputs: list[str] = []

    prereqs = run_command([*SELF_CLI, "ops", "prereqs", "--report", "text"], repo, ctx=ctx)
    if prereqs.combined_output.strip():
        outputs.append(prereqs.combined_output.rstrip())
    if prereqs.code != 0:
        return _emit_ops_status(report_format, prereqs.code, "\n".join(outputs).strip())

    outputs.append("evidence root: artifacts/evidence")
    outputs.append("evidence run id pointer: artifacts/evidence/latest-run-id.txt")
    env_summary = environment_summary(ctx, ["docker", "kind", "kubectl", "helm", "k6", "python3"])
    outputs.append("tool presence summary:")
    outputs.append(json.dumps({
        "required_tools": env_summary.get("required_tools", []),
        "missing_tools": env_summary.get("missing_tools", []),
        "tool_versions": env_summary.get("tool_versions", {}),
    }, sort_keys=True))

    best_effort = [
        ["python3", "./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py", "kind", "kubectl", "helm", "k6", "jq", "yq", "python3"],
        ["python3", "./packages/atlasctl/src/atlasctl/layout_checks/check_ops_pins.py"],
        ["make", "-s", "ops-env-print"],
    ]
    for cmd in best_effort:
        result = run_command(cmd, repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())

    legacy_pattern = re.compile(r"(?:legacy/[A-Za-z0-9_.-]+|ops-[A-Za-z0-9-]+-legacy|ops/.*/_legacy/|ops/.*/scripts/.*legacy)")
    legacy_hits: list[str] = []
    for rel in ["makefiles", "docs", ".github/workflows"]:
        base = repo / rel
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue
            for lineno, line in enumerate(text.splitlines(), start=1):
                if legacy_pattern.search(line):
                    legacy_hits.append(f"{path.relative_to(repo).as_posix()}:{lineno}:{line.strip()}")
    if legacy_hits:
        msg = "\n".join(outputs + ["legacy ops path/target references found in public surfaces", *legacy_hits]).strip()
        return _emit_ops_status(report_format, 1, msg)

    pin_report = repo / "artifacts" / "evidence" / "pins" / ctx.run_id / "pin-drift-report.json"
    if pin_report.exists():
        outputs.append(f"pin drift report: {pin_report.relative_to(repo).as_posix()}")
        outputs.append(pin_report.read_text(encoding="utf-8", errors="ignore").rstrip())

    return _emit_ops_status(report_format, 0, "\n".join(x for x in outputs if x).strip())


def _ops_smoke_native(ctx: RunContext, report_format: str, reuse: bool) -> int:
    repo = ctx.repo_root
    run_id = ctx.run_id
    start = dt.datetime.now(dt.timezone.utc)
    reuse_val = "1" if reuse else "0"
    outputs: list[str] = []
    log_dir = repo / "artifacts" / "evidence" / "ops-smoke" / run_id
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / "run.log"

    missing = [tool for tool in ("kind", "kubectl", "helm", "k6") if shutil.which(tool) is None]
    if missing:
        return _emit_ops_status(report_format, 1, "\n".join(f"missing required tool: {tool}" for tool in missing))

    steps: list[tuple[list[str], dict[str, str]]] = [
        (["env", f"REUSE={reuse_val}", "make", "-s", "ops-up"], {}),
        (["make", "-s", "ops-deploy"], {}),
        (["make", "-s", "ops-warm"], {}),
        (["make", "-s", "ops-api-smoke"], {}),
        (["env", "OBS_SKIP_LOCAL_COMPOSE=1", "SUITE=contracts", "make", "-s", "ops-obs-verify"], {}),
    ]

    status = "pass"
    down_ran = False
    try:
        for cmd, _ in steps:
            result = run_command(cmd, repo, ctx=ctx)
            if result.combined_output.strip():
                outputs.append(result.combined_output.rstrip())
            if result.code != 0:
                status = "fail"
                break
        if status == "pass":
            down = run_command(["make", "-s", "ops-down"], repo, ctx=ctx)
            down_ran = True
            if down.combined_output.strip():
                outputs.append(down.combined_output.rstrip())
            if down.code != 0:
                status = "fail"
    finally:
        if not down_ran:
            run_command(["make", "-s", "ops-down"], repo, ctx=ctx)

    write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
    duration_seconds = max(0.0, (dt.datetime.now(dt.timezone.utc) - start).total_seconds())
    lane_report = {
        "schema_version": 1,
        "run_id": run_id,
        "status": status,
        "duration_seconds": duration_seconds,
        "log": log_file.relative_to(repo).as_posix(),
        "repro_command": f"make ops/smoke REUSE={reuse_val}",
    }
    write_text_file(log_dir / "report.json", json.dumps(lane_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    run_command([*SELF_CLI, "report", "unified", "--run-id", run_id, "--out", "ops/_generated_committed/report.unified.json"], repo, ctx=ctx)
    if status == "pass":
        budget = run_command(
            ["env", f"RUN_ID={run_id}", "python3", "./packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py"],
            repo,
            ctx=ctx,
        )
        if budget.combined_output.strip():
            outputs.append(budget.combined_output.rstrip())
        if budget.code != 0:
            status = "fail"
    return _emit_ops_status(report_format, 0 if status == "pass" else 1, "\n".join(outputs).strip())


def _ops_run_migrated_shell_asset(ctx: RunContext, report_format: str, asset_name: str) -> int:
    asset = Path(__file__).resolve().parent / "assets" / asset_name
    if not asset.exists():
        return _emit_ops_status(report_format, 2, f"missing atlasctl ops asset: {asset_name}")
    return _run_simple_cmd(ctx, ["bash", str(asset)], report_format)


def _ops_warm_dx(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    cmds = [
        [*SELF_CLI, "ops", "warm", "--report", "text", "--mode", "datasets"],
        [*SELF_CLI, "ops", "warm", "--report", "text", "--mode", "shards"],
        [*SELF_CLI, "ops", "cache", "--report", "text", "status"],
    ]
    outputs: list[str] = []
    for cmd in cmds:
        result = run_command(cmd, repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
        if result.code != 0:
            return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())
    out_dir = repo / "artifacts" / "evidence" / "warm" / ctx.run_id
    out_dir.mkdir(parents=True, exist_ok=True)
    cache_report = repo / "artifacts" / "ops" / "cache-status" / "report.json"
    if cache_report.exists():
        shutil.copy2(cache_report, out_dir / "cache-status.json")
    write_text_file(out_dir / "summary.txt", f"warm completed run_id={ctx.run_id}\n", encoding="utf-8")
    msg = f"{out_dir.relative_to(repo).as_posix()}"
    if outputs:
        msg = "\n".join(outputs + [msg])
    return _emit_ops_status(report_format, 0, msg)


def _ops_warm_native(ctx: RunContext, report_format: str, mode: str) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []

    # Keep parity with prior shell preflight.
    for cmd in (["python3", "./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py", "kind", "kubectl"],):
        result = run_command(cmd, repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
        if result.code != 0:
            return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())

    dataset_profile = os.environ.get("ATLAS_DATASET_PROFILE", "")
    if not dataset_profile:
        return _emit_ops_status(
            report_format,
            2,
            "ATLAS_DATASET_PROFILE is required by configs/ops/env.schema.json",
        )
    profile = os.environ.get("PROFILE", "kind")
    if profile == "ci":
        dataset_profile = "ci"

    if dataset_profile == "ci":
        os.environ.setdefault("ATLAS_ALLOW_PRIVATE_STORE_HOSTS", "0")
        os.environ.setdefault("ATLAS_E2E_ENABLE_OTEL", os.environ.get("ATLAS_E2E_ENABLE_OTEL", "0"))
    elif dataset_profile == "developer":
        result = run_command([*SELF_CLI, "ops", "cache", "--report", "text", "status", "--plan"], repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
    else:
        return _emit_ops_status(report_format, 2, f"invalid ATLAS_DATASET_PROFILE={dataset_profile} (expected ci|developer)")

    guard_profile = "kind" if profile in {"ci", "developer"} else profile
    ctx_check = run_command(["kubectl", "config", "current-context"], repo, ctx=ctx)
    current_ctx = (ctx_check.combined_output or "").strip().splitlines()
    current_ctx_val = current_ctx[-1].strip() if current_ctx else ""
    context_ok = bool(current_ctx_val) and (guard_profile != "kind" or ("kind" in current_ctx_val))
    if not context_ok and guard_profile == "kind":
        outputs.append("ops-warm-stage: context missing; bootstrapping stack with reuse")
        result = run_command([*SELF_CLI, "ops", "stack", "--report", "text", "up", "--reuse", "--profile", guard_profile], repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
        if result.code != 0:
            return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())
    elif not context_ok:
        return _emit_ops_status(report_format, 2, f"invalid kubectl context for profile={guard_profile}")

    mode_to_script = {
        "warmup": "packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warmup.py",
        "datasets": "packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warm_datasets.py",
        "top": "packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warm_top.py",
        "shards": "packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warm_shards.py",
    }
    target = mode_to_script.get(mode)
    if not target:
        return _emit_ops_status(report_format, 2, f"invalid --mode={mode} (expected warmup|datasets|top|shards)")

    result = run_command([*SELF_CLI, "run", f"./{target}"], repo, ctx=ctx)
    if result.combined_output.strip():
        outputs.append(result.combined_output.rstrip())
    return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())


def _ops_obs_verify(ctx: RunContext, report_format: str, suite: str, extra_args: list[str]) -> int:
    repo = ctx.repo_root
    run_id = ctx.run_id
    log_dir = repo / "artifacts" / "evidence" / "obs-verify" / run_id
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / "run.log"
    start = dt.datetime.now(dt.timezone.utc)
    result = run_command(
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/observability/test_suite.py", "--suite", suite, *extra_args],
        repo,
        ctx=ctx,
    )
    write_text_file(log_file, result.combined_output if result.combined_output.endswith("\n") else result.combined_output + "\n", encoding="utf-8")
    status = "pass" if result.code == 0 else "fail"
    # Best-effort conformance report generation to preserve old wrapper side effects.
    run_command(
        [
            "python3",
            "ops/obs/scripts/areas/contracts/write_obs_conformance_report.py",
            "--run-id",
            run_id,
            "--suite",
            suite,
            "--status",
            status,
            "--out-dir",
            str(log_dir),
        ],
        repo,
        ctx=ctx,
    )
    traces = repo / "artifacts" / "ops" / "obs" / "traces.exemplars.log"
    if traces.exists():
        shutil.copy2(traces, log_dir / "traces.exemplars.log")
    if status == "pass":
        last_pass = repo / "artifacts" / "evidence" / "obs" / "last-pass.json"
        last_pass.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "run_id": run_id,
            "suite": suite,
            "timestamp_utc": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        }
        write_text_file(last_pass, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    duration_seconds = max(0.0, (dt.datetime.now(dt.timezone.utc) - start).total_seconds())
    lane_report = {
        "schema_version": 1,
        "run_id": run_id,
        "status": status,
        "duration_seconds": duration_seconds,
        "log": log_file.relative_to(repo).as_posix(),
    }
    write_text_file(log_dir / "report.json", json.dumps(lane_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    msg = str(log_dir.relative_to(repo))
    return _emit_ops_status(report_format, 0 if status == "pass" else 1, msg)


def _ops_undeploy_native(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []

    if shutil.which("helm") is None:
        return _emit_ops_status(report_format, 1, "missing required tool: helm")

    layer_contract = load_layer_contract(repo)
    ns = os.environ.get("ATLAS_E2E_NAMESPACE") or os.environ.get("ATLAS_NS") or lc_ns_e2e(layer_contract)
    release = os.environ.get("ATLAS_E2E_RELEASE_NAME") or lc_release_default(layer_contract)

    result = run_command(["helm", "-n", ns, "uninstall", release], repo, ctx=ctx)
    if result.combined_output.strip():
        outputs.append(result.combined_output.rstrip())
    outputs.append(f"undeploy complete: release={release} namespace={ns}")
    # Preserve previous behavior: uninstall failures were ignored.
    return _emit_ops_status(report_format, 0, "\n".join(outputs).strip())


def _ops_k8s_restart_native(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []

    if shutil.which("kubectl") is None:
        return _emit_ops_status(report_format, 1, "missing required tool: kubectl")

    layer_contract = load_layer_contract(repo)

    ns = (
        os.environ.get("ATLAS_E2E_NAMESPACE")
        or os.environ.get("ATLAS_NS")
        or lc_ns_k8s(layer_contract)
    )
    release = os.environ.get("ATLAS_E2E_RELEASE_NAME") or lc_release_default(layer_contract)
    service_name = os.environ.get("ATLAS_E2E_SERVICE_NAME") or lc_service_atlas(layer_contract)
    timeout = os.environ.get("ATLAS_E2E_TIMEOUT") or "180s"

    validate = run_command([*SELF_CLI, "ops", "k8s", "--report", "text", "validate-configmap-keys", ns, service_name], repo, ctx=ctx)
    if validate.combined_output.strip():
        outputs.append(validate.combined_output.rstrip())
    if validate.code != 0:
        return _emit_ops_status(report_format, validate.code, "\n".join(outputs).strip())

    outputs.append(f"rolling restart deployment/{service_name} in namespace {ns}")
    for cmd in (
        ["kubectl", "-n", ns, "rollout", "restart", f"deployment/{service_name}"],
        ["kubectl", "-n", ns, "rollout", "status", f"deployment/{service_name}", f"--timeout={timeout}"],
        ["kubectl", "-n", ns, "get", "deploy", service_name, "-o", "jsonpath={.status.readyReplicas}"],
    ):
        result = run_command(cmd, repo, ctx=ctx)
        out = result.combined_output.strip()
        if out:
            outputs.append(out)
        if result.code != 0:
            return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())
        if cmd[3:5] == ["get", "deploy"]:
            ready = out.strip()
            if not re.fullmatch(r"[1-9][0-9]*", ready):
                outputs.append(f"deployment {service_name} has non-ready replica count: {ready or '<empty>'}")
                return _emit_ops_status(report_format, 1, "\n".join(outputs).strip())

    outputs.append(f"k8s restart passed (ns={ns} release={release} service={service_name})")
    return _emit_ops_status(report_format, 0, "\n".join(outputs).strip())


def _ops_k8s_validate_configmap_keys_native(ctx: RunContext, report_format: str, namespace: str | None, service_name: str | None) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []

    strict_mode = os.environ.get("ATLAS_STRICT_CONFIG_KEYS", "1")
    if strict_mode != "1":
        return _emit_ops_status(
            report_format,
            0,
            f"configmap strict key validation skipped (ATLAS_STRICT_CONFIG_KEYS={strict_mode})",
        )

    missing = [tool for tool in ("helm", "kubectl") if shutil.which(tool) is None]
    if missing:
        return _emit_ops_status(report_format, 1, "\n".join(f"missing required tool: {tool}" for tool in missing))

    layer_contract = load_layer_contract(repo)
    ns = (
        namespace
        or os.environ.get("ATLAS_E2E_NAMESPACE")
        or os.environ.get("ATLAS_NS")
        or lc_ns_k8s(layer_contract)
    )
    svc = service_name or os.environ.get("ATLAS_E2E_SERVICE_NAME") or lc_service_atlas(layer_contract)
    cm_name = f"{svc}-config"
    values_file = (
        os.environ.get("ATLAS_E2E_VALUES_FILE")
        or os.environ.get("ATLAS_VALUES_FILE")
        or "./ops/k8s/values/local.yaml"
    )

    templ = run_command(
        ["helm", "template", svc, "./ops/k8s/charts/bijux-atlas", "-n", ns, "-f", values_file],
        repo,
        ctx=ctx,
    )
    if templ.code != 0 and templ.combined_output.strip():
        outputs.append(templ.combined_output.rstrip())
    if templ.code != 0:
        return _emit_ops_status(report_format, templ.code, "\n".join(outputs).strip())

    tmpl_keys: set[str] = set()
    for doc in templ.stdout.split("\n---"):
        if "kind: ConfigMap" not in doc:
            continue
        if f"name: {cm_name}" not in doc and "metadata:" in doc:
            # Prefer exact target configmap if present; skip other ConfigMaps.
            continue
        in_data = False
        for raw in doc.splitlines():
            line = raw.rstrip("\n")
            if re.match(r"^data:\s*$", line.strip()):
                in_data = True
                continue
            if in_data:
                if line and not line.startswith(" "):
                    in_data = False
                    continue
                m = re.match(r"^\s+(ATLAS_[A-Z0-9_]+):\s*", line)
                if m:
                    tmpl_keys.add(m.group(1))
        if tmpl_keys:
            break

    live = run_command(["kubectl", "-n", ns, "get", "configmap", cm_name, "-o", "json"], repo, ctx=ctx)
    if live.code != 0 and live.combined_output.strip():
        outputs.append(live.combined_output.rstrip())
    if live.code != 0:
        return _emit_ops_status(report_format, live.code, "\n".join(outputs).strip())
    try:
        live_payload = json.loads(live.stdout or "{}")
    except json.JSONDecodeError as exc:
        outputs.append(f"failed to parse kubectl configmap json: {exc}")
        return _emit_ops_status(report_format, 1, "\n".join(outputs).strip())
    live_data = live_payload.get("data", {}) if isinstance(live_payload, dict) else {}
    live_keys = sorted(k for k in live_data.keys() if isinstance(k, str))

    unknown = [k for k in live_keys if k not in tmpl_keys]
    if unknown:
        outputs.append(f"unknown configmap keys detected in {ns}/{cm_name}:")
        outputs.extend(unknown)
        return _emit_ops_status(report_format, 1, "\n".join(outputs).strip())

    outputs.append(f"configmap key validation passed ({ns}/{cm_name})")
    return _emit_ops_status(report_format, 0, "\n".join(outputs).strip())


def _ops_k8s_apply_config_native(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []

    missing = [tool for tool in ("kubectl", "helm") if shutil.which(tool) is None]
    if missing:
        return _emit_ops_status(report_format, 1, "\n".join(f"missing required tool: {tool}" for tool in missing))

    layer_contract = load_layer_contract(repo)
    ns = (
        os.environ.get("ATLAS_E2E_NAMESPACE")
        or os.environ.get("ATLAS_NS")
        or lc_ns_k8s(layer_contract)
    )
    service_name = os.environ.get("ATLAS_E2E_SERVICE_NAME") or lc_service_atlas(layer_contract)
    cm_name = f"{service_name}-config"
    values_file = (
        os.environ.get("ATLAS_E2E_VALUES_FILE")
        or os.environ.get("ATLAS_VALUES_FILE")
        or str((repo / "ops/k8s/values/local.yaml").resolve())
    )
    profile = os.environ.get("PROFILE") or "local"

    def _configmap_resource_version() -> tuple[int, str]:
        res = run_command(
            ["kubectl", "-n", ns, "get", "configmap", cm_name, "-o", "jsonpath={.metadata.resourceVersion}"],
            repo,
            ctx=ctx,
        )
        if res.code != 0:
            return 0, ""
        return 0, res.combined_output.strip()

    _, old_hash = _configmap_resource_version()

    values_validate = run_command(["make", "-s", "ops-values-validate"], repo, ctx=ctx)
    if values_validate.combined_output.strip():
        outputs.append(values_validate.combined_output.rstrip())
    if values_validate.code != 0:
        return _emit_ops_status(report_format, values_validate.code, "\n".join(outputs).strip())

    deploy = run_command(["env", f"PROFILE={profile}", "make", "-s", "ops-deploy"], repo, ctx=ctx)
    if deploy.combined_output.strip():
        outputs.append(deploy.combined_output.rstrip())
    if deploy.code != 0:
        return _emit_ops_status(report_format, deploy.code, "\n".join(outputs).strip())

    _, new_hash = _configmap_resource_version()
    if old_hash and new_hash and old_hash != new_hash:
        outputs.append(f"configmap changed after deploy ({cm_name}); restarting workloads")
        restart = run_command([*SELF_CLI, "ops", "restart", "--report", "text"], repo, ctx=ctx)
        if restart.combined_output.strip():
            outputs.append(restart.combined_output.rstrip())
        if restart.code != 0:
            return _emit_ops_status(report_format, restart.code, "\n".join(outputs).strip())
    else:
        outputs.append(f"configmap unchanged after deploy ({cm_name}); restart skipped")

    validate = run_command([*SELF_CLI, "ops", "k8s", "--report", "text", "validate-configmap-keys", ns, service_name], repo, ctx=ctx)
    if validate.combined_output.strip():
        outputs.append(validate.combined_output.rstrip())
    if validate.code != 0:
        return _emit_ops_status(report_format, validate.code, "\n".join(outputs).strip())

    outputs.append(f"k8s apply-config passed (values={values_file})")
    return _emit_ops_status(report_format, 0, "\n".join(outputs).strip())


def _ops_e2e_run_native(ctx: RunContext, report_format: str, suite: str) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []

    prereq = run_command(
        ["python3", "./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py", "python3", "kubectl", "helm"],
        repo,
        ctx=ctx,
    )
    if prereq.combined_output.strip():
        outputs.append(prereq.combined_output.rstrip())
    if prereq.code != 0:
        return _emit_ops_status(report_format, prereq.code, "\n".join(outputs).strip())

    result = run_command(
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/suite_runner.py", "--suite", str(suite)],
        repo,
        ctx=ctx,
    )
    if result.combined_output.strip():
        outputs.append(result.combined_output.rstrip())
    return _emit_ops_status(report_format, result.code, "\n".join(outputs).strip())


def _ops_stack_up_native(ctx: RunContext, report_format: str, profile: str, reuse: bool) -> int:
    repo = ctx.repo_root
    run_id = ctx.run_id
    start = dt.datetime.now(dt.timezone.utc)
    outputs: list[str] = []
    log_dir = repo / "artifacts" / "evidence" / "stack" / run_id
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / "stack-up.log"
    health_json = log_dir / "health-report.json"
    snapshot_json = repo / "artifacts" / "evidence" / "stack" / "state-snapshot.json"

    missing = [tool for tool in ("kind", "kubectl", "helm") if shutil.which(tool) is None]
    if missing:
        return _emit_ops_status(report_format, 1, "\n".join(f"missing required tool: {tool}" for tool in missing))

    env_code, env_msg, resolved = _ops_env_validate_native(repo, "configs/ops/env.schema.json")
    outputs.append(env_msg.strip())
    if env_code != 0:
        write_text_file(log_file, "\n".join(outputs).strip() + "\n", encoding="utf-8")
        return _emit_ops_status(report_format, env_code, "\n".join(outputs).strip())
    atlas_ns = (os.environ.get("ATLAS_E2E_NAMESPACE") or resolved.get("ATLAS_E2E_NAMESPACE") or "").strip()
    atlas_cluster = (os.environ.get("ATLAS_E2E_CLUSTER_NAME") or resolved.get("ATLAS_E2E_CLUSTER_NAME") or "").strip()
    if not atlas_ns or not atlas_cluster:
        missing_envs = []
        if not atlas_ns:
            missing_envs.append("ATLAS_E2E_NAMESPACE")
        if not atlas_cluster:
            missing_envs.append("ATLAS_E2E_CLUSTER_NAME")
        outputs.append(f"missing required ops env values: {', '.join(missing_envs)}")
        write_text_file(log_file, "\n".join(outputs).strip() + "\n", encoding="utf-8")
        return _emit_ops_status(report_format, 1, "\n".join(outputs).strip())

    guard = run_command([*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/context_guard.py"], repo, ctx=ctx)
    if reuse and snapshot_json.exists() and guard.code == 0:
        ns_check = run_command(["kubectl", "get", "ns", atlas_ns], repo, ctx=ctx)
        health = run_command(
            [
                "env",
                "ATLAS_HEALTH_REPORT_FORMAT=json",
                "python3",
                "./packages/atlasctl/src/atlasctl/commands/ops/stack/health_report.py",
                atlas_ns,
                str(health_json),
            ],
            repo,
            ctx=ctx,
        )
        if ns_check.code == 0 and health.code == 0:
            msg = f"stack-up reuse hit: healthy snapshot validated for namespace={atlas_ns}"
            write_text_file(log_file, msg + "\n", encoding="utf-8")
            lane_report = {
                "schema_version": 1,
                "run_id": run_id,
                "status": "pass",
                "duration_seconds": max(0.0, (dt.datetime.now(dt.timezone.utc) - start).total_seconds()),
                "log": log_file.relative_to(repo).as_posix(),
            }
            write_text_file(log_dir / "report.json", json.dumps(lane_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            return _emit_ops_status(report_format, 0, msg)

    status = "pass"
    cmds: list[list[str]] = [
        ["make", "-s", "ops-kind-up"],
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/context_guard.py"],
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/namespace_guard.py"],
        ["make", "-s", "ops-kind-version-check"],
        ["make", "-s", "ops-kubectl-version-check"],
        ["make", "-s", "ops-helm-version-check"],
    ]
    if os.environ.get("ATLAS_KIND_REGISTRY_ENABLE", "0") == "1":
        cmds.append(["make", "-s", "ops-kind-registry-up"])
    cmds.extend(
        [
            [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/install.py"],
            ["make", "-s", "ops-cluster-sanity"],
        ]
    )
    for cmd in cmds:
        result = run_command(cmd, repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
        if result.code != 0:
            status = "fail"
            break

    run_command(
        [
            "env",
            "ATLAS_HEALTH_REPORT_FORMAT=json",
            "python3",
            "./packages/atlasctl/src/atlasctl/commands/ops/stack/health_report.py",
            atlas_ns,
            str(health_json),
        ],
        repo,
        ctx=ctx,
    )

    write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
    duration_seconds = max(0.0, (dt.datetime.now(dt.timezone.utc) - start).total_seconds())
    lane_report = {
        "schema_version": 1,
        "run_id": run_id,
        "status": status,
        "duration_seconds": duration_seconds,
        "log": log_file.relative_to(repo).as_posix(),
    }
    write_text_file(log_dir / "report.json", json.dumps(lane_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if status == "pass":
        snapshot_json.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema_version": 1,
            "captured_at": dt.datetime.now(dt.timezone.utc).isoformat(),
            "profile": profile,
            "cluster": atlas_cluster,
            "namespace": atlas_ns,
            "health_report": health_json.relative_to(repo).as_posix(),
            "run_id": run_id,
            "healthy": True,
        }
        write_text_file(snapshot_json, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return _emit_ops_status(report_format, 0 if status == "pass" else 1, "\n".join(outputs).strip())


def _ops_stack_down_native(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    run_id = ctx.run_id
    start = dt.datetime.now(dt.timezone.utc)
    outputs: list[str] = []
    log_dir = repo / "artifacts" / "evidence" / "stack" / run_id
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / "stack-down.log"
    health_json = log_dir / "health-report-after-down.json"
    snapshot_json = repo / "artifacts" / "evidence" / "stack" / "state-snapshot.json"

    missing = [tool for tool in ("kind", "kubectl", "helm") if shutil.which(tool) is None]
    if missing:
        return _emit_ops_status(report_format, 1, "\n".join(f"missing required tool: {tool}" for tool in missing))

    env_code, env_msg, resolved = _ops_env_validate_native(repo, "configs/ops/env.schema.json")
    outputs.append(env_msg.strip())
    if env_code != 0:
        write_text_file(log_file, "\n".join(outputs).strip() + "\n", encoding="utf-8")
        return _emit_ops_status(report_format, env_code, "\n".join(outputs).strip())
    atlas_ns = (os.environ.get("ATLAS_E2E_NAMESPACE") or resolved.get("ATLAS_E2E_NAMESPACE") or "").strip()
    if not atlas_ns:
        outputs.append("ATLAS_E2E_NAMESPACE is required by configs/ops/env.schema.json")
        write_text_file(log_file, "\n".join(outputs).strip() + "\n", encoding="utf-8")
        return _emit_ops_status(report_format, 1, "\n".join(outputs).strip())

    status = "pass"
    for cmd in (
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/context_guard.py"],
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/namespace_guard.py"],
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/uninstall.py"],
    ):
        result = run_command(cmd, repo, ctx=ctx)
        if result.combined_output.strip():
            outputs.append(result.combined_output.rstrip())
        if result.code != 0:
            status = "fail"
            break

    run_command(
        [
            "env",
            "ATLAS_HEALTH_REPORT_FORMAT=json",
            "python3",
            "./packages/atlasctl/src/atlasctl/commands/ops/stack/health_report.py",
            atlas_ns,
            str(health_json),
        ],
        repo,
        ctx=ctx,
    )
    write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
    duration_seconds = max(0.0, (dt.datetime.now(dt.timezone.utc) - start).total_seconds())
    lane_report = {
        "schema_version": 1,
        "run_id": run_id,
        "status": status,
        "duration_seconds": duration_seconds,
        "log": log_file.relative_to(repo).as_posix(),
    }
    write_text_file(log_dir / "report.json", json.dumps(lane_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if status == "pass":
        snapshot_json.unlink(missing_ok=True)
    return _emit_ops_status(report_format, 0 if status == "pass" else 1, "\n".join(outputs).strip())


def _ops_deploy_native(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    outputs: list[str] = []

    missing = [tool for tool in ("kind", "kubectl", "helm", "python3") if shutil.which(tool) is None]
    if missing:
        return _emit_ops_status(report_format, 1, "\n".join(f"missing required tool: {tool}" for tool in missing))

    profile = os.environ.get("PROFILE") or "kind"
    guard_cmd = [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/context_guard.py"]
    guard = run_command(guard_cmd, repo, ctx=ctx)
    if guard.code != 0 and profile == "kind":
        outputs.append("ops-deploy: kind context missing; bootstrapping stack")
        stack_up = run_command([*SELF_CLI, "ops", "stack", "--report", "text", "up", "--profile", "kind"], repo, ctx=ctx)
        if stack_up.combined_output.strip():
            outputs.append(stack_up.combined_output.rstrip())
        if stack_up.code != 0:
            return _emit_ops_status(report_format, stack_up.code, "\n".join(outputs).strip())
        guard = run_command(guard_cmd, repo, ctx=ctx)
    if guard.combined_output.strip():
        outputs.append(guard.combined_output.rstrip())
    if guard.code != 0:
        return _emit_ops_status(report_format, guard.code, "\n".join(outputs).strip())

    deploy = run_command(
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/deploy_atlas.py"],
        repo,
        ctx=ctx,
    )
    if deploy.combined_output.strip():
        outputs.append(deploy.combined_output.rstrip())
    return _emit_ops_status(report_format, deploy.code, "\n".join(outputs).strip())


def _ops_load_run_native(ctx: RunContext, report_format: str, suite: str, out_dir: str = "artifacts/perf/results") -> int:
    repo = ctx.repo_root
    run_id = ctx.run_id
    start = dt.datetime.now(dt.timezone.utc)
    outputs: list[str] = []
    log_dir = repo / "artifacts" / "evidence" / "load-suite" / run_id
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / "run.log"
    out_path = (repo / out_dir) if not Path(out_dir).is_absolute() else Path(out_dir)
    out_path.mkdir(parents=True, exist_ok=True)

    profile = os.environ.get("PROFILE", "kind")
    if profile == "kind":
        guard = run_command([*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/context_guard.py"], repo, ctx=ctx)
        if guard.combined_output.strip():
            outputs.append(guard.combined_output.rstrip())
        if guard.code != 0:
            write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
            return _emit_ops_status(report_format, guard.code, "\n".join(outputs).strip())

    # Preserve prior behavior: require k6 via tool check (docker fallback only for execution image).
    k6_check = run_command(["python3", "./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py", "k6"], repo, ctx=ctx)
    if k6_check.combined_output.strip():
        outputs.append(k6_check.combined_output.rstrip())
    if k6_check.code != 0:
        write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
        return _emit_ops_status(report_format, k6_check.code, "\n".join(outputs).strip())

    input_name = str(suite)
    if input_name.endswith(".json"):
        scenario_path = Path(input_name)
        if not scenario_path.is_absolute():
            scenario_path = repo / "ops" / "load" / "scenarios" / input_name
        scenario_payload = json.loads(scenario_path.read_text(encoding="utf-8"))
        resolved_suite = str(scenario_payload.get("suite", "")).strip()
        if not resolved_suite:
            outputs.append(f"scenario has no suite: {scenario_path}")
            write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
            return _emit_ops_status(report_format, 2, "\n".join(outputs).strip())
        suite_name = scenario_path.stem
    else:
        resolved_suite = input_name if input_name.endswith(".js") else f"{input_name}.js"
        suite_name = Path(resolved_suite).stem

    manifest = json.loads((repo / "ops/load/suites/suites.json").read_text(encoding="utf-8"))
    declared = {str(row.get("name", "")).strip() for row in manifest.get("suites", []) if isinstance(row, dict)}
    if suite_name not in declared:
        msg = f"adhoc suite forbidden: {suite_name} is not declared in ops/load/suites/suites.json"
        outputs.append(msg)
        write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
        return _emit_ops_status(report_format, 1, "\n".join(outputs).strip())

    tools_versions = json.loads((repo / "configs/ops/tool-versions.json").read_text(encoding="utf-8"))
    k6_version = str(tools_versions.get("k6", "v1.0.0")).lstrip("v")

    base_url = os.environ.get("ATLAS_BASE_URL") or os.environ.get("BASE_URL") or "http://127.0.0.1:18080"
    api_key = os.environ.get("ATLAS_API_KEY", "")
    dataset_hash = os.environ.get("ATLAS_DATASET_HASH", "unknown")
    dataset_release = os.environ.get("ATLAS_DATASET_RELEASE", "unknown")
    image_digest = os.environ.get("ATLAS_IMAGE_DIGEST", "unknown")
    git_sha = os.environ.get("GITHUB_SHA", "")
    if not git_sha:
        git_res = run_command(["git", "rev-parse", "--short=12", "HEAD"], repo, ctx=ctx)
        git_sha = git_res.stdout.strip() if git_res.code == 0 and git_res.stdout.strip() else "unknown"
    policy_hash = os.environ.get("ATLAS_POLICY_HASH", "")
    if not policy_hash:
        policy_file = repo / "configs/policy/policy.json"
        if policy_file.exists():
            policy_hash = hashlib.sha256(policy_file.read_bytes()).hexdigest()
        else:
            policy_hash = "unknown"

    summary_json = out_path / f"{suite_name}.summary.json"
    suite_script = f"ops/load/k6/suites/{resolved_suite}"
    if shutil.which("k6") is not None:
        k6_cmd = ["env", f"BASE_URL={base_url}", f"ATLAS_API_KEY={api_key}", "k6", "run", "--summary-export", str(summary_json), suite_script]
    else:
        try:
            summary_export = str(summary_json.relative_to(repo))
        except ValueError:
            outputs.append(f"docker k6 fallback requires --out under repo root: {summary_json}")
            write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
            return _emit_ops_status(report_format, 2, "\n".join(outputs).strip())
        k6_cmd = [
            "docker", "run", "--rm", "--network", "host",
            "-e", f"BASE_URL={base_url}",
            "-e", f"ATLAS_API_KEY={api_key}",
            "-v", f"{repo}:/work", "-w", "/work",
            f"grafana/k6:{k6_version}", "run", "--summary-export", summary_export, suite_script,
        ]
    k6_run = run_command(k6_cmd, repo, ctx=ctx)
    if k6_run.combined_output.strip():
        outputs.append(k6_run.combined_output.rstrip())
    status = "pass" if k6_run.code == 0 else "fail"

    meta_json = out_path / f"{suite_name}.meta.json"
    meta_payload = {
        "suite": input_name,
        "resolved_suite": resolved_suite,
        "git_sha": git_sha,
        "image_digest": image_digest,
        "dataset_hash": dataset_hash,
        "dataset_release": dataset_release,
        "policy_hash": policy_hash,
        "base_url": base_url,
    }
    write_text_file(meta_json, json.dumps(meta_payload, sort_keys=True) + "\n", encoding="utf-8")

    evidence_raw = repo / "artifacts" / "evidence" / "perf" / run_id / "raw"
    evidence_raw.mkdir(parents=True, exist_ok=True)
    if summary_json.exists():
        shutil.copy2(summary_json, evidence_raw / f"{suite_name}.summary.json")
    if meta_json.exists():
        shutil.copy2(meta_json, evidence_raw / f"{suite_name}.meta.json")
    outputs.append(f"suite complete: {input_name} ({resolved_suite}) -> {summary_json}")

    write_text_file(log_file, ("\n".join(outputs).strip() + "\n") if outputs else "", encoding="utf-8")
    duration_seconds = max(0.0, (dt.datetime.now(dt.timezone.utc) - start).total_seconds())
    lane_report = {
        "schema_version": 1,
        "run_id": run_id,
        "status": status,
        "duration_seconds": duration_seconds,
        "log": log_file.relative_to(repo).as_posix(),
    }
    write_text_file(log_dir / "report.json", json.dumps(lane_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return _emit_ops_status(report_format, 0 if status == "pass" else 1, "\n".join(outputs).strip())


def _ops_refgrade_audit_native(ctx: RunContext, report_format: str, *, strict: bool = False) -> int:
    checks = [
        "checks_ops_surface_manifest",
        "checks_ops_suites_contracts",
        "checks_ops_report_contract_fields",
    ]
    rows: list[dict[str, object]] = []
    ok = True
    for check_id in checks:
        proc = run_command(["./bin/atlasctl", "check", "run", "--id", check_id, "--quiet"], ctx.repo_root, ctx=ctx)
        rows.append({"id": check_id, "code": proc.code})
        ok = ok and proc.code == 0
    for name, cmd in (
        ("ops.schema-check", ["./bin/atlasctl", "ops", "schema-check", "--report", "text"]),
        ("ops.pins.check", ["./bin/atlasctl", "ops", "pins", "check", "--report", "text"]),
        ("policies.bypass.inventory", ["./bin/atlasctl", "policies", "culprits", "--format", "json"]),
    ):
        proc = run_command(cmd, ctx.repo_root, ctx=ctx)
        rows.append({"id": name, "code": proc.code})
        ok = ok and proc.code == 0
    if strict:
        proc = run_command(["./bin/atlasctl", "policies", "bypass", "drill", "--strict", "--report", "json"], ctx.repo_root, ctx=ctx)
        rows.append({"id": "policies.bypass.drill.strict", "code": proc.code})
        ok = ok and proc.code == 0
    payload = {
        "schema_version": 1,
        "kind": "ops-refgrade-audit-scorecard",
        "run_id": ctx.run_id,
        "status": "pass" if ok else "fail",
        "strict": strict,
        "checks": rows,
    }
    out = ctx.repo_root / "ops/_generated_committed/scorecard.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
        return 0 if ok else 1
    return _emit_ops_status(report_format, 0 if ok else 1, "ops-refgrade-audit: wrote ops/_generated_committed/scorecard.json")





def run_ops_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    from importlib import import_module

    run = getattr(import_module("atlasctl.commands.ops.runtime_modules.ops_runtime_run"), "run_ops_command")

    return run(ctx, ns)


def configure_ops_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    from importlib import import_module

    configure = getattr(import_module("atlasctl.commands.ops.runtime_modules.ops_runtime_parser"), "configure_ops_parser")

    configure(sub)
