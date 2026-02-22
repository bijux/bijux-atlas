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





def run_ops_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    from importlib import import_module

    run = getattr(import_module("atlasctl.commands.ops.runtime_modules.ops_runtime_run"), "run_ops_command")

    return run(ctx, ns)


def configure_ops_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    from importlib import import_module

    configure = getattr(import_module("atlasctl.commands.ops.runtime_modules.ops_runtime_parser"), "configure_ops_parser")

    configure(sub)
