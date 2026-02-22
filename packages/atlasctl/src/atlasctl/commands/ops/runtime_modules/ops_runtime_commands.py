from __future__ import annotations

import json
import os
import re
import shutil
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.core.process import run_command
from atlasctl.core.runtime.paths import write_text_file

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
