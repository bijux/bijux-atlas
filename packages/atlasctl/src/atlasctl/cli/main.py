from __future__ import annotations

import argparse
import importlib
import json
import os
import sys
from datetime import date
from pathlib import Path

from .. import __version__, registry
from ..checks.repo import layout
from ..cli.surface_registry import command_registry, register_domain_parser
from ..core.context import RunContext
from ..core.runtime.env import getenv, setdefault as env_setdefault, setenv
from ..core.exec import check_output
from ..core.effects import command_effects, command_group
from ..core.fs import write_json, write_text
from ..core.runtime.logging import log_event
from ..core.runtime.telemetry import emit_telemetry
from ..core.runtime.repo_root import try_find_repo_root
from ..contracts.ids import COMMANDS, HELP, RUNTIME_CONTRACTS
from ..core.errors import ScriptError
from ..core.exit_codes import ERR_CONFIG, ERR_INTERNAL
from ..core.runtime.guards.network_guard import install_no_network_guard, resolve_network_mode
from .constants import CONFIGURE_HOOKS, DOMAINS, NO_NETWORK_FLAG_EXPIRY
from .dispatch import dispatch_command
from .help_formatter import format_public_help
from .output import no_network_flag_expired, render_error, resolve_output_format

DOMAIN_RUNNERS = {"registry": registry.run, "layout": layout.run}
_PUBLIC_GROUPS: tuple[tuple[str, str], ...] = (
    ("docs", "documentation and docs contracts"),
    ("configs", "configuration validation and sync"),
    ("dev", "development checks, suites, and tests"),
    ("ops", "operations and runtime orchestration"),
    ("policies", "policy enforcement and culprits"),
    ("internal", "internal/legacy compatibility and diagnostics"),
)


def _import_attr(module_name: str, attr: str):
    return getattr(importlib.import_module(module_name), attr)


def _version_string() -> str:
    base = f"atlasctl {__version__}"
    try:
        repo_root = try_find_repo_root()
        if repo_root is None:
            return f"{base}+unknown"
        sha = check_output(["git", "rev-parse", "--short", "HEAD"], cwd=repo_root).strip()
        if sha:
            return f"{base}+{sha}"
    except Exception:
        pass
    return f"{base}+unknown"


def _write_payload_if_requested(ctx: RunContext, out_file: str | None, payload: str) -> None:
    if out_file:
        write_text(ctx, Path(out_file), payload + "\n")


def _commands_payload(include_internal: bool = False) -> dict[str, object]:
    return {
        "schema_name": COMMANDS,
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "ok": True,
        "errors": [],
        "warnings": [],
        "meta": {"include_internal": include_internal},
        "run_id": "",
        "commands": [
            {
                "name": cmd.name,
                "help": cmd.help_text,
                "stable": cmd.stable,
                "touches": list(cmd.touches),
                "tools": list(cmd.tools),
                "failure_modes": list(cmd.failure_modes),
                "owner": cmd.owner,
                "doc_link": cmd.doc_link,
                "effect_level": cmd.effect_level,
                "run_id_mode": cmd.run_id_mode,
                "supports_dry_run": cmd.supports_dry_run,
                "aliases": list(cmd.aliases),
                "purpose": cmd.purpose or cmd.help_text,
                "examples": list(cmd.examples),
                "internal": cmd.internal,
            }
            for cmd in sorted(command_registry(), key=lambda item: item.name)
            if include_internal or not cmd.internal
        ],
    }


def _ensure_scripts_artifact_root(ctx: RunContext) -> Path:
    root = ctx.scripts_artifact_root
    env_setdefault("BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT", str(root))
    (root / "reports").mkdir(parents=True, exist_ok=True)
    (root / "logs").mkdir(parents=True, exist_ok=True)
    return root


def _emit_runtime_contracts(ctx: RunContext, cmd: str, argv: list[str] | None) -> None:
    root = _ensure_scripts_artifact_root(ctx)
    write_roots = {
        "schema_name": RUNTIME_CONTRACTS,
        "schema_version": 1,
        "tool": "atlasctl",
        "run_id": ctx.run_id,
        "status": "ok",
        "allowed_write_roots": [str(ctx.evidence_root), str(root), str(root / "reports"), str(root / "logs")],
        "forbidden_roots": [str(ctx.repo_root / path) for path in ("ops", "docs", "configs", "makefiles", "crates")],
    }
    run_manifest = {
        "schema_version": 1,
        "tool": "atlasctl",
        "run_id": ctx.run_id,
        "command": cmd,
        "argv": argv or [],
        "generated_at": ctx.run_id,
        "git_sha": ctx.git_sha,
        "git_dirty": ctx.git_dirty,
        "repo_root": str(ctx.repo_root),
        "artifact_root": str(root),
        "command_group": command_group(cmd),
        "declared_effects": list(command_effects(cmd)),
        "network_mode": ctx.network_mode,
        "network_requested": bool("--allow-network" in (argv or [])),
    }
    write_json(ctx, root / "reports" / "write-roots-contract.json", write_roots)
    write_json(ctx, root / "reports" / "run-manifest.json", run_manifest)


def _apply_python_env(ctx: RunContext) -> None:
    env_setdefault("BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT", str(ctx.scripts_root))
    env_setdefault("ATLASCTL_ARTIFACT_ROOT", str(ctx.scripts_root))
    env_setdefault("XDG_CACHE_HOME", str((ctx.scripts_root / "cache").resolve()))
    env_setdefault("PYTHONPYCACHEPREFIX", str((ctx.scripts_root / "pycache").resolve()))
    env_setdefault("MYPY_CACHE_DIR", str((ctx.scripts_root / "mypy").resolve()))
    env_setdefault("RUFF_CACHE_DIR", str((ctx.scripts_root / "ruff").resolve()))
    env_setdefault("PIP_CACHE_DIR", str((ctx.scripts_root / "pip").resolve()))
    env_setdefault("UV_CACHE_DIR", str((ctx.scripts_root / "pip").resolve()))
    required = f"--cache-dir={(ctx.scripts_root / 'pytest').resolve()}"
    existing = getenv("PYTEST_ADDOPTS", "").strip()
    if required not in existing.split():
        setenv("PYTEST_ADDOPTS", f"{existing} {required}".strip())


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="atlasctl")
    parser.add_argument("--version", action="version", version=_version_string())
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    parser.add_argument("--run-id", help="run identifier for artifacts")
    parser.add_argument("--dry-run", action="store_true", help="resolve and report command without mutating side effects")
    parser.add_argument("--artifacts-dir", help="artifacts/evidence root path")
    parser.add_argument("--evidence-root", help="evidence root path")
    parser.add_argument("--run-dir", help="override run directory root")
    parser.add_argument("--cwd", help="run command from an explicit repository root")
    parser.add_argument("--profile", help="profile id")
    parser.add_argument("--format", choices=["text", "json"], default=None, help="output format")
    parser.add_argument("--log-json", action="store_true", help="emit structured JSON logs to stderr")
    parser.add_argument("--network", choices=["allow", "forbid"], default=None, help="network access mode (default: forbid)")
    parser.add_argument("--allow-network", action="store_true", help="explicitly allow network when command group policy permits it")
    parser.add_argument("--no-network", action="store_true", help="deprecated alias for --network=forbid")
    parser.add_argument("--require-clean-git", action="store_true", help="fail if git workspace is dirty")
    vg = parser.add_mutually_exclusive_group()
    vg.add_argument("--verbose", action="store_true", help="enable verbose diagnostics")
    vg.add_argument("--quiet", action="store_true", help="only emit errors")
    sub = parser.add_subparsers(dest="cmd", required=True)

    version_parser = sub.add_parser("version", help="print versions and git context")
    version_parser.add_argument("--json", action="store_true", help="emit JSON output")
    for module_name, attr in CONFIGURE_HOOKS[:2]:
        _import_attr(module_name, attr)(sub)
    self_parser = sub.add_parser("self-check", help="validate imports, config loading, and schema presence")
    self_parser.add_argument("--json", action="store_true", help="emit JSON output")
    help_parser = sub.add_parser("help", help="print command help")
    help_parser.add_argument("--json", action="store_true", help="emit machine-readable command inventory")
    help_parser.add_argument("--out-file", help="optional output path")
    help_parser.add_argument("--include-internal", action="store_true", help=argparse.SUPPRESS)
    explain_parser = sub.add_parser("explain", help="describe command contracts and usage")
    explain_parser.add_argument("subject_or_name")
    explain_parser.add_argument("name", nargs="?")
    explain_parser.add_argument("--json", action="store_true", help="emit JSON output")
    _import_attr("atlasctl.commands.internal.listing", "configure_list_parser")(sub)
    run_id_parser = sub.add_parser("run-id", help="generate and print a deterministic atlasctl run id")
    run_id_parser.add_argument("--prefix", default="atlas", help="run id prefix")
    run_id_parser.add_argument("--json", action="store_true", help="emit JSON output")

    run_parser = sub.add_parser("run", help="run an internal python script by repo-relative path")
    run_parser.add_argument("script")
    run_parser.add_argument("args", nargs=argparse.REMAINDER)
    run_parser.add_argument("--dry-run", action="store_true", help="print resolved command and exit")

    validate_parser = sub.add_parser("validate-output", help="validate JSON output against schema")
    validate_parser.add_argument("--schema", required=True)
    validate_parser.add_argument("--file", required=True)
    validate_parser.add_argument("--json", action="store_true", help="emit JSON output")

    surface_parser = sub.add_parser("surface", help="print scripts command ownership surface")
    surface_parser.add_argument("--json", action="store_true", help="emit JSON output")
    surface_parser.add_argument("--out-file", help="optional output path for JSON report")
    commands_parser = sub.add_parser("commands", help="print machine-readable command surface")
    commands_parser.add_argument("commands_cmd", nargs="?", choices=["list", "lint"], default="list")
    commands_parser.add_argument("--json", action="store_true", help="emit JSON output")
    commands_parser.add_argument("--out-file", help="optional output path for JSON report")
    commands_parser.add_argument("--verify-stability", action="store_true", help="compare command payload against commands golden")
    commands_parser.add_argument("--include-internal", action="store_true", help=argparse.SUPPRESS)

    config_parser = sub.add_parser("config", help="configuration commands (alias over `configs`)")
    config_sub = config_parser.add_subparsers(dest="config_cmd", required=True)
    config_sub.add_parser("dump", help="dump canonical config payload").add_argument("--report", choices=["text", "json"], default="json")
    config_sub.add_parser("validate", help="validate config schemas and policy").add_argument("--report", choices=["text", "json"], default="text")
    config_sub.add_parser("drift", help="check generated config drift").add_argument("--report", choices=["text", "json"], default="text")

    for name in DOMAINS:
        register_domain_parser(sub, name, f"{name} domain commands")
    for module_name, attr in CONFIGURE_HOOKS[2:]:
        _import_attr(module_name, attr)(sub)

    sub.add_parser("doctor", help="show tooling and context diagnostics").add_argument("--json", action="store_true", help="emit JSON output")
    completion_parser = sub.add_parser("completion", help="emit shell completion stub")
    completion_parser.add_argument("shell", choices=["bash", "zsh", "fish"])
    completion_parser.add_argument("--json", action="store_true", help="emit JSON output")
    clean_parser = sub.add_parser("clean", help="clean scripts artifacts under approved roots only")
    clean_parser.add_argument("--older-than-days", type=int)
    clean_parser.add_argument("--json", action="store_true", help="emit JSON output")
    fix_parser = sub.add_parser("fix", help="run explicit fixers (separate from checks)")
    fix_parser.add_argument("thing", nargs="?", default="list", help="fixer id or `list`")
    fix_parser.add_argument("--json", action="store_true", help="emit JSON output")

    raw_format_help = parser.format_help

    def _format_help_public() -> str:
        return format_public_help(raw_format_help(), _PUBLIC_GROUPS)

    parser.format_help = _format_help_public  # type: ignore[assignment]
    return parser


def main(argv: list[str] | None = None) -> int:
    # LEGACY entrypoint facade. Cutover target: 2026-04-01.
    from ..app.main import main as app_main

    return app_main(argv)


if __name__ == "__main__":
    raise SystemExit(main())
