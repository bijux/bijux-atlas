from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import sys
from pathlib import Path

from . import contracts, layout, registry
from .compat.command import configure_compat_parser, run_compat_command
from .ci.command import configure_ci_parser, run_ci_command
from .check.command import configure_check_parser, run_check_command
from .configs.command import configure_configs_parser, run_configs_command
from .core.context import RunContext
from .core.env_guard import guard_no_network_mode
from .core.fs import ensure_evidence_path
from .core.logging import log_event
from .docs.command import configure_docs_parser, run_docs_command
from .docker.command import configure_docker_parser, run_docker_command
from .doctor import run_doctor
from .domain_cmd import register_domain_parser, render_payload
from .domain_cmd import registry as command_registry
from .env.command import clean_scripts_artifacts, configure_env_parser, run_env_command
from .errors import ScriptError
from .exit_codes import ERR_CONFIG, ERR_INTERNAL
from .gates.command import configure_gates_parser, run_gates_command
from .gen.command import configure_gen_parser, run_gen_command
from .inventory.command import configure_inventory_parser, run_inventory
from .legacy.command import configure_legacy_parser, run_legacy_command
from .lint.command import configure_lint_parser, run_lint_command
from .make.command import configure_make_parser, run_make_command
from .network_guard import install_no_network_guard
from .ops.command import configure_ops_parser, run_ops_command
from .orchestrate.command import configure_orchestrate_parsers, run_orchestrate_command
from .output_contract import validate_json_output
from .policies.command import configure_policies_parser, run_policies_command
from .report.command import configure_report_parser, run_report_command
from .runner import run_legacy_script
from .surface import run_surface

DOMAINS = {
    "contracts": contracts.run,
    "registry": registry.run,
    "layout": layout.run,
}


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="bijux-atlas")
    p.add_argument("--version", action="version", version=_version_string())
    p.add_argument("--run-id", help="run identifier for artifacts")
    p.add_argument("--evidence-root", help="evidence root path")
    p.add_argument("--run-dir", help="override run directory root")
    p.add_argument("--profile", help="profile id")
    p.add_argument("--format", choices=["text", "json"], default=None, help="output format")
    p.add_argument("--network", choices=["allow", "forbid"], default="allow", help="network access mode")
    p.add_argument("--no-network", action="store_true", help="deprecated alias for --network=forbid")
    p.add_argument("--require-clean-git", action="store_true", help="fail if git workspace is dirty")
    vg = p.add_mutually_exclusive_group()
    vg.add_argument("--verbose", action="store_true", help="enable verbose diagnostics")
    vg.add_argument("--quiet", action="store_true", help="only emit errors")
    sub = p.add_subparsers(dest="cmd", required=True)

    version_p = sub.add_parser("version", help="print versions and git context")
    version_p.add_argument("--json", action="store_true", help="emit JSON output")
    configure_env_parser(sub)
    self_p = sub.add_parser("self-check", help="validate imports, config loading, and schema presence")
    self_p.add_argument("--json", action="store_true", help="emit JSON output")
    help_p = sub.add_parser("help", help="print command help")
    help_p.add_argument("--json", action="store_true", help="emit machine-readable command inventory")
    help_p.add_argument("--out-file", help="optional output path")

    run_p = sub.add_parser("run", help="run an internal python script by repo-relative path")
    run_p.add_argument("script")
    run_p.add_argument("args", nargs=argparse.REMAINDER)
    run_p.add_argument("--dry-run", action="store_true", help="print resolved command and exit")

    val_p = sub.add_parser("validate-output", help="validate JSON output against schema")
    val_p.add_argument("--schema", required=True)
    val_p.add_argument("--file", required=True)
    val_p.add_argument("--json", action="store_true", help="emit JSON output")

    surface_p = sub.add_parser("surface", help="print scripts command ownership surface")
    surface_p.add_argument("--json", action="store_true", help="emit JSON output")
    surface_p.add_argument("--out-file", help="optional output path for JSON report")
    commands_p = sub.add_parser("commands", help="print machine-readable command surface")
    commands_p.add_argument("--json", action="store_true", help="emit JSON output")
    commands_p.add_argument("--out-file", help="optional output path for JSON report")

    domain_names = ("contracts", "registry", "layout")
    for name in domain_names:
        register_domain_parser(sub, name, f"{name} domain commands")
    configure_configs_parser(sub)
    configure_docker_parser(sub)
    configure_ci_parser(sub)
    configure_check_parser(sub)
    configure_gen_parser(sub)
    configure_policies_parser(sub)
    configure_docs_parser(sub)
    configure_make_parser(sub)
    configure_ops_parser(sub)
    configure_inventory_parser(sub)
    configure_lint_parser(sub)
    configure_report_parser(sub)
    configure_compat_parser(sub)
    configure_legacy_parser(sub)
    configure_orchestrate_parsers(sub)
    configure_gates_parser(sub)

    doctor_p = sub.add_parser("doctor", help="show tooling and context diagnostics")
    doctor_p.add_argument("--json", action="store_true", help="emit JSON output")
    doctor_p.add_argument("--out-file", help="optional output path for JSON report")

    completion_p = sub.add_parser("completion", help="emit shell completion stub")
    completion_p.add_argument("shell", choices=["bash", "zsh", "fish"])
    completion_p.add_argument("--json", action="store_true", help="emit JSON output")

    clean_p = sub.add_parser("clean", help="clean scripts artifacts under approved roots only")
    clean_p.add_argument("--older-than-days", type=int)
    clean_p.add_argument("--json", action="store_true", help="emit JSON output")

    return p


def _version_string() -> str:
    base = "bijux-atlas 0.1.0"
    try:
        repo_root = Path(__file__).resolve().parents[4]
        sha = (
            subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], cwd=repo_root, text=True)
            .strip()
        )
        if sha:
            return f"{base}+{sha}"
    except Exception:
        pass
    return f"{base}+unknown"


def _write_payload_if_requested(ctx: RunContext, out_file: str | None, payload: str) -> None:
    if not out_file:
        return
    out_path = ensure_evidence_path(ctx, Path(out_file))
    out_path.write_text(payload + "\n", encoding="utf-8")


def _emit(payload: dict[str, object], as_json: bool) -> None:
    if as_json:
        print(json.dumps(payload, sort_keys=True))
    else:
        print(json.dumps(payload, indent=2, sort_keys=True))


def _commands_payload() -> dict[str, object]:
    return {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "commands": [
            {"name": c.name, "help": c.help_text, "stable": c.stable}
            for c in sorted(command_registry(), key=lambda c: c.name)
        ],
    }


def _build_common_payload(ctx: RunContext, status: str = "ok") -> dict[str, object]:
    return {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": status,
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        "repo_root": str(ctx.repo_root),
        "run_dir": str(ctx.run_dir),
        "evidence_root": str(ctx.evidence_root),
        "scripts_artifact_root": str(ctx.scripts_artifact_root),
        "network": ctx.network_mode,
        "format": ctx.output_format,
        "git_sha": ctx.git_sha,
        "git_dirty": ctx.git_dirty,
    }


def _ensure_scripts_artifact_root(ctx: RunContext) -> Path:
    root = ctx.scripts_artifact_root
    os.environ.setdefault("BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT", str(root))
    (root / "reports").mkdir(parents=True, exist_ok=True)
    (root / "logs").mkdir(parents=True, exist_ok=True)
    return root


def _emit_runtime_contracts(ctx: RunContext, cmd: str, argv: list[str] | None) -> None:
    root = _ensure_scripts_artifact_root(ctx)
    write_roots = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "allowed_write_roots": [str(ctx.evidence_root), str(root), str(root / "reports"), str(root / "logs")],
        "forbidden_roots": [str(ctx.repo_root / p) for p in ("ops", "docs", "configs", "makefiles", "crates")],
    }
    run_manifest = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "command": cmd,
        "argv": argv or [],
        "generated_at": ctx.run_id,
        "git_sha": ctx.git_sha,
        "git_dirty": ctx.git_dirty,
        "host": platform.node(),
        "repo_root": str(ctx.repo_root),
        "artifact_root": str(root),
    }
    (root / "reports" / "write-roots-contract.json").write_text(
        json.dumps(write_roots, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (root / "reports" / "run-manifest.json").write_text(
        json.dumps(run_manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main(argv: list[str] | None = None) -> int:
    p = build_parser()
    ns = p.parse_args(argv)
    fmt = ns.format or ("json" if "CI" in os.environ else "text")
    ctx = RunContext.from_args(
        ns.run_id,
        ns.evidence_root,
        ns.profile,
        ns.no_network,
        fmt,
        ns.network,
        ns.run_dir,
        ns.verbose,
        ns.quiet,
        ns.require_clean_git,
    )
    _emit_runtime_contracts(ctx, ns.cmd, argv)
    restore_network = None
    if ctx.no_network:
        guard_no_network_mode(True)
        restore_network = install_no_network_guard()
    try:
        if ctx.require_clean_git and ctx.git_dirty:
            raise ScriptError("git workspace is dirty; rerun without --require-clean-git or commit changes", ERR_CONFIG)
        if not ctx.quiet:
            log_event(ctx, "info", "cli", "start", cmd=ns.cmd, fmt=ctx.output_format, network=ctx.network_mode)
        as_json = ctx.output_format == "json" or bool(getattr(ns, "json", False))
        if ns.cmd == "version":
            _emit(
                {
                    **_build_common_payload(ctx),
                    "scripts_version": _version_string().split()[1],
                },
                as_json,
            )
            return 0
        if ns.cmd == "env":
            return run_env_command(ctx, ns)
        if ns.cmd == "self-check":
            payload = _build_common_payload(ctx)
            payload["checks"] = {
                "imports": "ok",
                "config_dir_exists": (ctx.repo_root / "configs").is_dir(),
                "schemas_dir_exists": (ctx.repo_root / "configs" / "_schemas").is_dir(),
            }
            payload["status"] = "ok" if all(payload["checks"].values()) else "fail"
            _emit(payload, as_json)
            return 0 if payload["status"] == "ok" else ERR_CONFIG
        if ns.cmd == "help":
            payload = _commands_payload()
            rendered = json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True)
            if ns.out_file:
                _write_payload_if_requested(ctx, ns.out_file, rendered)
            print(rendered)
            return 0
        if ns.cmd == "completion":
            payload = {"schema_version": 1, "tool": "bijux-atlas", "shell": ns.shell, "status": "ok"}
            if as_json:
                print(json.dumps(payload, sort_keys=True))
            else:
                print(f"# completion for {ns.shell} is not yet generated; use `bijux-atlas help --json`")
            return 0
        if ns.cmd == "clean":
            payload = clean_scripts_artifacts(ctx, ns.older_than_days)
            if as_json or ns.json:
                print(json.dumps(payload, sort_keys=True))
            else:
                print(f"removed={len(payload.get('removed', []))}")
            return 0
        if ns.cmd == "run":
            if ns.dry_run:
                _emit(
                    {
                        "schema_version": 1,
                        "tool": "bijux-atlas",
                        "status": "ok",
                        "script": ns.script,
                        "args": ns.args,
                    },
                    as_json,
                )
                return 0
            return run_legacy_script(ns.script, ns.args, ctx)
        if ns.cmd == "validate-output":
            return validate_json_output(ns.schema, ns.file, ns.json)
        if ns.cmd == "surface":
            return run_surface(ns.json, ns.out_file)
        if ns.cmd == "commands":
            return run_surface(True, ns.out_file)
        if ns.cmd == "doctor":
            return run_doctor(ctx, ns.json, ns.out_file)
        if ns.cmd == "docs":
            return run_docs_command(ctx, ns)
        if ns.cmd == "configs":
            return run_configs_command(ctx, ns)
        if ns.cmd == "docker":
            return run_docker_command(ctx, ns)
        if ns.cmd == "ci":
            return run_ci_command(ctx, ns)
        if ns.cmd == "check":
            return run_check_command(ctx, ns)
        if ns.cmd == "gen":
            return run_gen_command(ctx, ns)
        if ns.cmd == "policies":
            return run_policies_command(ctx, ns)
        if ns.cmd == "make":
            return run_make_command(ctx, ns)
        if ns.cmd == "ops":
            return run_ops_command(ctx, ns)
        if ns.cmd == "inventory":
            return run_inventory(ctx, ns.category, ns.format, ns.out_dir, ns.dry_run, ns.check)
        if ns.cmd == "report":
            return run_report_command(ctx, ns)
        if ns.cmd == "lint":
            return run_lint_command(ctx, ns)
        if ns.cmd == "compat":
            return run_compat_command(ctx, ns)
        if ns.cmd == "legacy":
            return run_legacy_command(ctx, ns)
        if ns.cmd in {"ports", "artifacts", "k8s", "stack", "obs", "load", "e2e", "datasets", "cleanup", "scenario"}:
            return run_orchestrate_command(ctx, ns)
        if ns.cmd == "gates":
            return run_gates_command(ctx, ns)
        if ns.cmd in DOMAINS:
            payload_obj = DOMAINS[ns.cmd](ctx)
            payload = render_payload(payload_obj, as_json)
            _write_payload_if_requested(ctx, ns.out_file, payload)
            print(payload)
            return 0
        return 2
    except ScriptError as exc:
        if ctx.output_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "bijux-atlas",
                        "status": "fail",
                        "error": {"message": str(exc), "code": exc.code},
                    },
                    sort_keys=True,
                ),
                file=sys.stderr,
            )
        else:
            print(str(exc), file=sys.stderr)
        return exc.code
    except Exception as exc:  # pragma: no cover
        if "ctx" in locals() and ctx.output_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "bijux-atlas",
                        "status": "fail",
                        "error": {"message": f"internal error: {exc}", "code": ERR_INTERNAL},
                    },
                    sort_keys=True,
                ),
                file=sys.stderr,
            )
        else:
            print(f"internal error: {exc}", file=sys.stderr)
        return ERR_INTERNAL
    finally:
        if restore_network:
            restore_network()


if __name__ == "__main__":
    raise SystemExit(main())
