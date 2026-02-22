from __future__ import annotations

import json
import os
import re
import shutil
import hashlib
import datetime as dt
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.core.process import run_command
from atlasctl.core.runtime.paths import write_text_file

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
    path = repo_root / "packages/atlasctl/src/atlasctl/registry/ops_tasks_catalog.json"
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = payload.get("tasks", []) if isinstance(payload, dict) else []
    catalog: dict[str, dict[str, str]] = {}
    for row in rows:
        if not isinstance(row, dict):
            continue
        name = str(row.get("name", "")).strip()
        if not name:
            continue
        catalog[name] = {
            "manifest": str(row.get("manifest", "")).strip(),
            "owner": str(row.get("owner", "")).strip(),
            "docs": str(row.get("docs", "")).strip(),
            "description": str(row.get("description", "")).strip(),
        }
    return catalog

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
    script = """
set -euo pipefail
cd .
. ./ops/_lib/common.sh
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-prereqs"
for c in docker kind kubectl helm k6 python3; do
  command -v "$c" >/dev/null 2>&1 || ops_fail "$OPS_ERR_PREREQ" "missing required tool: $c"
done
ops_version_guard kind kubectl helm k6
python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py kind kubectl helm k6 jq yq python3
python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_ops_pins.py
python3 --version
kubectl version --client >/dev/null
helm version --short >/dev/null
kind version >/dev/null
k6 version >/dev/null
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_doctor_native(ctx: RunContext, report_format: str) -> int:
    script = """
set -euo pipefail
cd .
. ./ops/_lib/common.sh
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-doctor"
ops_version_guard kind kubectl helm k6
./bin/atlasctl ops prereqs --report text
echo "evidence root: artifacts/evidence"
echo "evidence run id pointer: artifacts/evidence/latest-run-id.txt"
python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py kind kubectl helm k6 jq yq python3 || true
python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_ops_pins.py || true
if rg -n "(?:legacy/[A-Za-z0-9_.-]+|ops-[A-Za-z0-9-]+-legacy|ops/.*/_legacy/|ops/.*/scripts/.*legacy)" \\
  makefiles docs .github/workflows >/dev/null 2>&1; then
  echo "legacy ops path/target references found in public surfaces" >&2
  rg -n "(?:legacy/[A-Za-z0-9_.-]+|ops-[A-Za-z0-9-]+-legacy|ops/.*/_legacy/|ops/.*/scripts/.*legacy)" \\
    makefiles docs .github/workflows || true
  exit 1
fi
pin_report="artifacts/evidence/pins/${RUN_ID}/pin-drift-report.json"
if [ -f "$pin_report" ]; then
  echo "pin drift report: $pin_report"
  cat "$pin_report"
fi
make -s ops-env-print || true
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_smoke_native(ctx: RunContext, report_format: str, reuse: bool) -> int:
    reuse_val = "1" if reuse else "0"
    script = f"""
set -euo pipefail
cd "{ctx.repo_root}"
. ./ops/_lib/common.sh
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-smoke"
ops_version_guard kind kubectl helm k6
start="$(date +%s)"
log_dir="artifacts/evidence/ops-smoke/${{RUN_ID}}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"
status="pass"
if ! (
  REUSE="{reuse_val}" make -s ops-up
  trap 'make -s ops-down >/dev/null 2>&1 || true' EXIT INT TERM
  make -s ops-deploy
  make -s ops-warm
  make -s ops-api-smoke
  OBS_SKIP_LOCAL_COMPOSE=1 SUITE=contracts make -s ops-obs-verify
  trap - EXIT INT TERM
  make -s ops-down
) >"$log_file" 2>&1; then
  status="fail"
fi
end="$(date +%s)"
duration="$((end - start))"
LANE_REPRO_COMMAND="make ops/smoke REUSE={reuse_val}" \\
ops_write_lane_report "ops-smoke" "${{RUN_ID}}" "${{status}}" "${{duration}}" "${{log_file}}" "artifacts/evidence" >/dev/null
./bin/atlasctl report unified --run-id "${{RUN_ID}}" --out ops/_generated_committed/report.unified.json >/dev/null
if [ "$status" = "pass" ]; then
  RUN_ID="${{RUN_ID}}" python3 ./packages/atlasctl/src/atlasctl/commands/ops/lint/policy/ops_smoke_budget_check.py
fi
[ "$status" = "pass" ] || exit 1
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


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
    script = f"""
set -euo pipefail
cd "{ctx.repo_root}"
. ./ops/_lib/common.sh
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-warm-stage"
ops_version_guard kind kubectl
mode="{mode}"
dataset_profile="${{ATLAS_DATASET_PROFILE:?ATLAS_DATASET_PROFILE is required by configs/ops/env.schema.json}}"
if [ "${{PROFILE:-kind}}" = "ci" ]; then
  dataset_profile="ci"
fi
case "$dataset_profile" in
  ci)
    export ATLAS_ALLOW_PRIVATE_STORE_HOSTS=0
    export ATLAS_E2E_ENABLE_OTEL="${{ATLAS_E2E_ENABLE_OTEL:-0}}"
    ;;
  developer)
    ./bin/atlasctl ops cache --report text status --plan || true
    ;;
  *)
    echo "invalid ATLAS_DATASET_PROFILE=${{dataset_profile}} (expected ci|developer)" >&2
    exit 2
    ;;
esac
profile="${{PROFILE:-kind}}"
guard_profile="$profile"
case "$guard_profile" in
  ci|developer) guard_profile="kind" ;;
esac
if ! ops_context_guard "$guard_profile"; then
  if [ "$guard_profile" = "kind" ]; then
    echo "ops-warm-stage: context missing; bootstrapping stack with reuse" >&2
    ./bin/atlasctl ops stack --report text up --reuse --profile "$guard_profile"
  fi
fi
case "$mode" in
  warmup) exec ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warmup.py ;;
  datasets) exec ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warm_datasets.py ;;
  top) exec ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warm_top.py ;;
  shards) exec ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warm_shards.py ;;
  *)
    echo "invalid --mode=${{mode}} (expected warmup|datasets|top|shards)" >&2
    exit 2
    ;;
esac
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_obs_verify(ctx: RunContext, report_format: str, suite: str, extra_args: list[str]) -> int:
    repo = ctx.repo_root
    run_id = ctx.run_id
    log_dir = repo / "artifacts" / "evidence" / "obs-verify" / run_id
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / "run.log"
    start = dt.datetime.now(dt.timezone.utc)
    result = run_command(["bash", "ops/obs/tests/suite.sh", "--suite", suite, *extra_args], repo, ctx=ctx)
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
    script = """
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-undeploy"
ops_version_guard helm
ns="${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-$(ops_layer_ns_e2e)}}"
release="${ATLAS_E2E_RELEASE_NAME:-$(ops_layer_contract_get release_metadata.defaults.release_name)}"
ops_helm -n "$ns" uninstall "$release" >/dev/null 2>&1 || true
echo "undeploy complete: release=$release namespace=$ns"
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_k8s_restart_native(ctx: RunContext, report_format: str) -> int:
    script = """
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "k8s-restart"
ops_version_guard kubectl
NS="${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-$(ops_layer_ns_k8s)}}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-$(ops_layer_contract_get release_metadata.defaults.release_name)}"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$(ops_layer_service_atlas)}"
TIMEOUT="${ATLAS_E2E_TIMEOUT:-180s}"
./bin/atlasctl ops k8s --report text validate-configmap-keys "$NS" "$SERVICE_NAME"
echo "rolling restart deployment/${SERVICE_NAME} in namespace ${NS}"
ops_kubectl -n "$NS" rollout restart deployment/"$SERVICE_NAME" >/dev/null
ops_kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout="$TIMEOUT" >/dev/null
ops_kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.readyReplicas}' | grep -Eq '^[1-9][0-9]*$'
echo "k8s restart passed (ns=${NS} release=${RELEASE} service=${SERVICE_NAME})"
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_k8s_validate_configmap_keys_native(ctx: RunContext, report_format: str, namespace: str | None, service_name: str | None) -> int:
    ns_arg = namespace or ""
    svc_arg = service_name or ""
    script = f"""
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-k8s-validate-configmap-keys"
ops_version_guard helm kubectl
NS="${{1:-${{ATLAS_E2E_NAMESPACE:-${{ATLAS_NS:-$(ops_layer_ns_k8s)}}}}}}"
SERVICE_NAME="${{2:-${{ATLAS_E2E_SERVICE_NAME:-$(ops_layer_service_atlas)}}}}"
CM_NAME="${{SERVICE_NAME}}-config"
STRICT_MODE="${{ATLAS_STRICT_CONFIG_KEYS:-1}}"
if [ "$STRICT_MODE" != "1" ]; then
  echo "configmap strict key validation skipped (ATLAS_STRICT_CONFIG_KEYS=${{STRICT_MODE}})"
  exit 0
fi
tmpl_keys="$(mktemp)"
live_keys="$(mktemp)"
trap 'rm -f "$tmpl_keys" "$live_keys"' EXIT
ops_helm template "$SERVICE_NAME" "./ops/k8s/charts/bijux-atlas" -n "$NS" -f "${{ATLAS_E2E_VALUES_FILE:-${{ATLAS_VALUES_FILE:-./ops/k8s/values/local.yaml}}}}" \\
  | awk '
    $0 ~ /^kind: ConfigMap$/ {{in_cm=1; next}}
    in_cm && $0 ~ /^metadata:/ {{next}}
    in_cm && $0 ~ /^data:/ {{in_data=1; next}}
    in_data && $1 ~ /^ATLAS_[A-Z0-9_]+:$/ {{gsub(":", "", $1); print $1}}
    in_data && $0 !~ /^[[:space:]]/ {{in_cm=0; in_data=0}}
  ' | sort -u > "$tmpl_keys"
ops_kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{{range $k,$v := .data}}{{${{k}}}}' 2>/dev/null | sort -u > "$live_keys"
unknown="$(comm -13 "$tmpl_keys" "$live_keys" || true)"
if [ -n "$unknown" ]; then
  echo "unknown configmap keys detected in ${{NS}}/${{CM_NAME}}:" >&2
  echo "$unknown" >&2
  exit 1
fi
echo "configmap key validation passed (${{NS}}/${{CM_NAME}})"
"""
    # jsonpath braces are difficult inside f-strings; keep exact behavior with positional args.
    script = script.replace("{{${k}}}", '{$k}{"\\n"}{end}')
    args = ["bash", "-lc", script, "bash"]
    if ns_arg:
        args.append(ns_arg)
    if svc_arg:
        args.append(svc_arg)
    return _run_simple_cmd(ctx, args, report_format)


def _ops_k8s_apply_config_native(ctx: RunContext, report_format: str) -> int:
    script = """
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "k8s-apply-config"
ops_version_guard kubectl helm
NS="${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-$(ops_layer_ns_k8s)}}"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$(ops_layer_service_atlas)}"
CM_NAME="${SERVICE_NAME}-config"
VALUES_FILE="${ATLAS_E2E_VALUES_FILE:-${ATLAS_VALUES_FILE:-$PWD/ops/k8s/values/local.yaml}}"
PROFILE="${PROFILE:-local}"
old_hash="$(ops_kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{.metadata.resourceVersion}' 2>/dev/null || true)"
make -s ops-values-validate
PROFILE="$PROFILE" make -s ops-deploy
new_hash="$(ops_kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{.metadata.resourceVersion}' 2>/dev/null || true)"
if [ -n "$old_hash" ] && [ -n "$new_hash" ] && [ "$old_hash" != "$new_hash" ]; then
  echo "configmap changed after deploy (${CM_NAME}); restarting workloads"
  ./bin/atlasctl ops restart --report text
else
  echo "configmap unchanged after deploy (${CM_NAME}); restart skipped"
fi
./bin/atlasctl ops k8s --report text validate-configmap-keys "$NS" "$SERVICE_NAME"
echo "k8s apply-config passed (values=${VALUES_FILE})"
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_e2e_run_native(ctx: RunContext, report_format: str, suite: str) -> int:
    script = f"""
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-e2e"
ops_version_guard python3 kubectl helm
exec ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/suite_runner.py --suite "{suite}"
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_stack_up_native(ctx: RunContext, report_format: str, profile: str, reuse: bool) -> int:
    reuse_flag = "1" if reuse else "0"
    script = f"""
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-stack-up"
ops_version_guard kind kubectl helm
reuse="{reuse_flag}"
profile="{profile}"
start_ts="$(date +%s)"
status="pass"
log_file="artifacts/evidence/stack/${{RUN_ID}}/stack-up.log"
health_json="artifacts/evidence/stack/${{RUN_ID}}/health-report.json"
snapshot_json="artifacts/evidence/stack/state-snapshot.json"
atlas_ns="${{ATLAS_E2E_NAMESPACE:?ATLAS_E2E_NAMESPACE is required by configs/ops/env.schema.json}}"
atlas_cluster="${{ATLAS_E2E_CLUSTER_NAME:?ATLAS_E2E_CLUSTER_NAME is required by configs/ops/env.schema.json}}"
mkdir -p "$(dirname "$log_file")"
if [ "$reuse" = "1" ] && [ -f "$snapshot_json" ]; then
  if ops_context_guard "$profile" >/dev/null 2>&1 \\
    && ops_kubectl get ns "$atlas_ns" >/dev/null 2>&1 \\
    && ATLAS_HEALTH_REPORT_FORMAT=json python3 ./packages/atlasctl/src/atlasctl/commands/ops/stack/health_report.py "$atlas_ns" "$health_json" >/dev/null 2>&1; then
    duration="$(( $(date +%s) - start_ts ))"
    ops_write_lane_report "stack" "$RUN_ID" "pass" "$duration" "$log_file"
    echo "stack-up reuse hit: healthy snapshot validated for namespace=${{atlas_ns}}" >"$log_file"
    exit 0
  fi
fi
if ! (
  make -s ops-env-validate
  make -s ops-kind-up
  ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/context_guard.py
  ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/namespace_guard.py
  make -s ops-kind-version-check
  make -s ops-kubectl-version-check
  make -s ops-helm-version-check
  if [ "${{ATLAS_KIND_REGISTRY_ENABLE:-0}}" = "1" ]; then make -s ops-kind-registry-up; fi
  ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/stack/install.py
  make -s ops-cluster-sanity
) >"$log_file" 2>&1; then
  status="fail"
fi
ATLAS_HEALTH_REPORT_FORMAT=json python3 ./packages/atlasctl/src/atlasctl/commands/ops/stack/health_report.py "$atlas_ns" "$health_json" >/dev/null || true
duration="$(( $(date +%s) - start_ts ))"
ops_write_lane_report "stack" "$RUN_ID" "$status" "$duration" "$log_file"
if [ "$status" = "pass" ]; then
  mkdir -p "$(dirname "$snapshot_json")"
  python3 - <<PY > "$snapshot_json"
import json, datetime
print(json.dumps({{
  "schema_version": 1,
  "captured_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
  "profile": "{profile}",
  "cluster": "${{atlas_cluster}}",
  "namespace": "${{atlas_ns}}",
  "health_report": "${{health_json}}",
  "run_id": "${{RUN_ID}}",
  "healthy": True
}}, indent=2, sort_keys=True))
PY
fi
[ "$status" = "pass" ] || exit 1
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_stack_down_native(ctx: RunContext, report_format: str) -> int:
    script = """
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-stack-down"
ops_version_guard kind kubectl helm
start_ts="$(date +%s)"
status="pass"
log_file="artifacts/evidence/stack/${RUN_ID}/stack-down.log"
health_json="artifacts/evidence/stack/${RUN_ID}/health-report-after-down.json"
snapshot_json="artifacts/evidence/stack/state-snapshot.json"
atlas_ns="${ATLAS_E2E_NAMESPACE:?ATLAS_E2E_NAMESPACE is required by configs/ops/env.schema.json}"
mkdir -p "$(dirname "$log_file")"
if ! (
  make -s ops-env-validate
  ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/context_guard.py
  ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/stack/kind/namespace_guard.py
  ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/stack/uninstall.py
) >"$log_file" 2>&1; then
  status="fail"
fi
ATLAS_HEALTH_REPORT_FORMAT=json python3 ./packages/atlasctl/src/atlasctl/commands/ops/stack/health_report.py "$atlas_ns" "$health_json" >/dev/null || true
duration="$(( $(date +%s) - start_ts ))"
ops_write_lane_report "stack" "$RUN_ID" "$status" "$duration" "$log_file"
[ "$status" = "pass" ] && rm -f "$snapshot_json"
[ "$status" = "pass" ] || exit 1
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_deploy_native(ctx: RunContext, report_format: str) -> int:
    script = """
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-deploy"
ops_version_guard kind kubectl helm python3
profile="${PROFILE:-kind}"
if ! ops_context_guard "$profile"; then
  if [ "$profile" = "kind" ]; then
    echo "ops-deploy: kind context missing; bootstrapping stack" >&2
    ./bin/atlasctl ops stack --report text up --profile kind
  fi
fi
ops_context_guard "$profile"
exec ./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/deploy_atlas.py
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)


def _ops_load_run_native(ctx: RunContext, report_format: str, suite: str, out_dir: str = "artifacts/perf/results") -> int:
    script = f"""
set -euo pipefail
. ./ops/_lib/common.sh
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-load-suite"
PROFILE="${{PROFILE:-kind}}"
ops_context_guard "$PROFILE"
ops_version_guard k6
INPUT="{suite}"
OUT="{out_dir}"
start="$(date +%s)"
log_dir="artifacts/evidence/load-suite/${{RUN_ID}}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"
status="pass"
if ! (
  python3 "./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py" k6 >/dev/null
  K6_VERSION="$(python3 - <<PY
import json
from pathlib import Path
data=json.loads(Path("configs/ops/tool-versions.json").read_text())
print(str(data.get("k6","v1.0.0")).lstrip("v"))
PY
)"
  BASE_URL="${{ATLAS_BASE_URL:-${{BASE_URL:-http://127.0.0.1:18080}}}}"
  API_KEY="${{ATLAS_API_KEY:-}}"
  DATASET_HASH="${{ATLAS_DATASET_HASH:-unknown}}"
  DATASET_RELEASE="${{ATLAS_DATASET_RELEASE:-unknown}}"
  IMAGE_DIGEST="${{ATLAS_IMAGE_DIGEST:-unknown}}"
  GIT_SHA="${{GITHUB_SHA:-$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)}}"
  POLICY_HASH="${{ATLAS_POLICY_HASH:-$(shasum -a 256 configs/policy/policy.json 2>/dev/null | awk '{{print $1}}' || echo unknown)}}"
  mkdir -p "$OUT"
  if printf '%s' "$INPUT" | grep -q '\\.json$'; then
    SCENARIO_PATH="$INPUT"
    case "$SCENARIO_PATH" in
      /*) ;;
      *) SCENARIO_PATH="ops/load/scenarios/$SCENARIO_PATH" ;;
    esac
    SUITE="$(python3 - <<PY
import json
from pathlib import Path
p=Path("$SCENARIO_PATH")
d=json.loads(p.read_text())
print(d.get("suite",""))
PY
)"
    [ -n "$SUITE" ] || {{ echo "scenario has no suite: $SCENARIO_PATH" >&2; exit 2; }}
    NAME="$(basename "$SCENARIO_PATH" .json)"
  else
    SUITE="$INPUT"
    NAME="${{SUITE%.js}}"
    case "$SUITE" in
      *.js) ;;
      *) SUITE="${{SUITE}}.js" ;;
    esac
  fi
  python3 - <<PY
import json
from pathlib import Path
manifest=json.loads(Path("ops/load/suites/suites.json").read_text())
names={{s.get("name","") for s in manifest.get("suites",[]) if isinstance(s,dict)}}
if "$NAME" not in names:
    raise SystemExit("adhoc suite forbidden: $NAME is not declared in ops/load/suites/suites.json")
PY
  SUMMARY_JSON="$OUT/${{NAME}}.summary.json"
  if command -v k6 >/dev/null 2>&1; then
    BASE_URL="$BASE_URL" ATLAS_API_KEY="$API_KEY" k6 run --summary-export "$SUMMARY_JSON" "ops/load/k6/suites/$SUITE"
  else
    docker run --rm --network host \
      -e BASE_URL="$BASE_URL" \
      -e ATLAS_API_KEY="$API_KEY" \
      -v "$PWD:/work" -w /work \
      "grafana/k6:${{K6_VERSION}}" run --summary-export "$SUMMARY_JSON" "ops/load/k6/suites/$SUITE"
  fi
  cat > "${{OUT}}/${{NAME}}.meta.json" <<JSON
{{"suite":"$INPUT","resolved_suite":"$SUITE","git_sha":"$GIT_SHA","image_digest":"$IMAGE_DIGEST","dataset_hash":"$DATASET_HASH","dataset_release":"$DATASET_RELEASE","policy_hash":"$POLICY_HASH","base_url":"$BASE_URL"}}
JSON
  RUN_REF="${{RUN_ID:-$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo manual)}}"
  EVIDENCE_RAW="artifacts/evidence/perf/$RUN_REF/raw"
  mkdir -p "$EVIDENCE_RAW"
  cp -f "$SUMMARY_JSON" "$EVIDENCE_RAW/${{NAME}}.summary.json"
  cp -f "${{OUT}}/${{NAME}}.meta.json" "$EVIDENCE_RAW/${{NAME}}.meta.json"
  echo "suite complete: $INPUT ($SUITE) -> $SUMMARY_JSON"
) >"$log_file" 2>&1; then
  status="fail"
fi
end="$(date +%s)"
ops_write_lane_report "load-suite" "${{RUN_ID}}" "${{status}}" "$((end - start))" "$log_file" "artifacts/evidence" >/dev/null
[ "$status" = "pass" ] || exit 1
"""
    return _run_simple_cmd(ctx, ["bash", "-lc", script], report_format)





def run_ops_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    from importlib import import_module

    run = getattr(import_module("atlasctl.commands.ops.runtime_modules.ops_runtime_run"), "run_ops_command")

    return run(ctx, ns)


def configure_ops_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    from importlib import import_module

    configure = getattr(import_module("atlasctl.commands.ops.runtime_modules.ops_runtime_parser"), "configure_ops_parser")

    configure(sub)
