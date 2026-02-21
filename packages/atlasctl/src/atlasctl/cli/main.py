from __future__ import annotations

import argparse
import importlib
import json
import os
import sys
from datetime import date
from pathlib import Path

from .. import __version__, layout, registry
from ..cli.surface_registry import command_registry, register_domain_parser
from ..core.context import RunContext
from ..core.env import getenv, setdefault as env_setdefault, setenv
from ..core.exec import check_output
from ..core.effects import command_effects, command_group
from ..core.fs import write_json, write_text
from ..core.logging import log_event
from ..core.telemetry import emit_telemetry
from ..core.repo_root import try_find_repo_root
from ..contracts.ids import COMMANDS, RUNTIME_CONTRACTS
from ..errors import ScriptError
from ..exit_codes import ERR_CONFIG, ERR_INTERNAL
from ..network_guard import install_no_network_guard, resolve_network_mode
from .constants import CONFIGURE_HOOKS, DOMAINS, NO_NETWORK_FLAG_EXPIRY
from .dispatch import dispatch_command
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
    _import_attr("atlasctl.commands.listing", "configure_list_parser")(sub)

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
        text = raw_format_help()
        lines = [line for line in text.splitlines() if "==SUPPRESS==" not in line]
        header: list[str] = []
        in_options = False
        for line in lines:
            if line.strip() == "options:":
                in_options = True
            if in_options:
                header.append(line)
        out = [
            "usage: atlasctl [global options] <group> ...",
            "",
            "control-plane groups:",
            *[f"  {name:<10} {desc}" for name, desc in _PUBLIC_GROUPS],
            "",
            "run `atlasctl <group> --help` for group commands.",
            "",
            *header,
        ]
        return "\n".join(out).rstrip() + "\n"

    parser.format_help = _format_help_public  # type: ignore[assignment]
    return parser


def main(argv: list[str] | None = None) -> int:
    raw_argv = argv if argv is not None else sys.argv[1:]
    parser = build_parser()
    ns = parser.parse_args(argv)
    if ns.format and "--json" in raw_argv and ns.format != "json":
        raise ScriptError("conflicting output flags: use either --format json or --json", ERR_CONFIG)
    if ns.cwd:
        os.chdir(ns.cwd)
    if ns.no_network and no_network_flag_expired(date.today(), NO_NETWORK_FLAG_EXPIRY):
        print("--no-network flag expired; use --network=forbid", file=sys.stderr)
        return ERR_CONFIG
    if ns.cmd == "version" and try_find_repo_root() is None:
        as_json = bool(getattr(ns, "json", False) or "--json" in raw_argv)
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "atlasctl_version": __version__, "scripts_version": _version_string().split()[1]}
        print(json.dumps(payload, sort_keys=True) if as_json else _version_string())
        return 0

    fmt = resolve_output_format(cli_json=("--json" in raw_argv), cli_format=ns.format, ci_present=bool(getenv("CI")))
    decision = resolve_network_mode(
        command_name=ns.cmd,
        requested_allow_network=bool(getattr(ns, "allow_network", False)),
        explicit_network=ns.network,
        deprecated_no_network=ns.no_network,
    )
    if getattr(ns, "allow_network", False) and not decision.allow_effective:
        raise ScriptError(
            f"--allow-network denied for command group `{decision.group}` (reason={decision.reason})",
            ERR_CONFIG,
        )
    ctx = RunContext.from_args(
        ns.run_id,
        ns.artifacts_dir or ns.evidence_root,
        ns.profile,
        decision.mode == "forbid",
        fmt,
        decision.mode,  # type: ignore[arg-type]
        ns.run_dir,
        ns.verbose,
        ns.quiet,
        ns.require_clean_git,
        ns.log_json,
    )
    _apply_python_env(ctx)
    _emit_runtime_contracts(ctx, ns.cmd, raw_argv)

    restore_network = None
    if ctx.no_network:
        from ..core.runtime.env_guard import guard_no_network_mode

        guard_no_network_mode(True)
        restore_network = install_no_network_guard()

    try:
        print(f"run_id={ctx.run_id}", file=sys.stderr)
        if ctx.require_clean_git and ctx.git_dirty:
            raise ScriptError("git workspace is dirty; rerun without --require-clean-git or commit changes", ERR_CONFIG)
        if decision.allow_effective and not ctx.quiet:
            print(
                f"NETWORK ENABLED: command={ns.cmd} group={decision.group} reason={decision.reason}",
                file=sys.stderr,
            )
        if not ctx.quiet:
            log_event(
                ctx,
                "info",
                "cli",
                "start",
                cmd=ns.cmd,
                fmt=ctx.output_format,
                network=ctx.network_mode,
                command_group=decision.group,
                declared_effects=list(command_effects(ns.cmd)),
                network_requested=decision.allow_requested,
                network_reason=decision.reason,
            )
        emit_telemetry(ctx, "cli.start", cmd=ns.cmd, output_format=ctx.output_format, network=ctx.network_mode)
        as_json = ctx.output_format == "json" or bool(getattr(ns, "json", False))
        rc = dispatch_command(ctx, ns, as_json, _import_attr, _commands_payload, _write_payload_if_requested, DOMAIN_RUNNERS, _version_string)
        emit_telemetry(ctx, "cli.finish", cmd=ns.cmd, rc=rc)
        return rc
    except ScriptError as exc:
        emit_telemetry(ctx, "cli.error", cmd=ns.cmd, kind=exc.kind, code=exc.code)
        print(
            render_error(
                as_json=(ctx.output_format == "json"),
                message=str(exc),
                code=exc.code,
                kind=exc.kind,
                run_id=ctx.run_id,
            ),
            file=sys.stderr,
        )
        return exc.code
    except Exception as exc:  # pragma: no cover
        if "ctx" in locals():
            emit_telemetry(ctx, "cli.internal_error", cmd=getattr(ns, "cmd", "unknown"), error=str(exc))
        print(
            render_error(
                as_json=("ctx" in locals() and ctx.output_format == "json"),
                message=f"internal error: {exc}",
                code=ERR_INTERNAL,
                run_id=(ctx.run_id if "ctx" in locals() else ""),
            ),
            file=sys.stderr,
        )
        return ERR_INTERNAL
    finally:
        if restore_network:
            restore_network()


if __name__ == "__main__":
    raise SystemExit(main())
