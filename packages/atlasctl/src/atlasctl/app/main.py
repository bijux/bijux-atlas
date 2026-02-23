from __future__ import annotations

import json
import os
import sys
from datetime import date

from .. import __version__
from ..cli.main import (
    DOMAIN_RUNNERS,
    _apply_python_env,
    _commands_payload,
    _emit_runtime_contracts,
    _import_attr,
    _version_string,
    _write_payload_if_requested,
    build_parser,
)
from ..cli.dispatch import dispatch_command
from ..cli.output import no_network_flag_expired, render_error, resolve_output_format
from ..core.context import RunContext
from ..core.effects import command_effects
from ..core.errors import ScriptError
from ..core.exit_codes import ERR_CONFIG, ERR_INTERNAL
from ..core.runtime.env import getenv
from ..core.runtime.guards.network_guard import install_no_network_guard, resolve_network_mode
from ..core.runtime.repo_root import try_find_repo_root
from ..core.runtime.telemetry import emit_telemetry
from ..contracts.ids import HELP
from ..cli.constants import NO_NETWORK_FLAG_EXPIRY
from ..runtime.logging import log_event


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
    if ns.cmd in {"help", "commands"} and try_find_repo_root() is None:
        as_json = bool(getattr(ns, "json", False) or "--json" in raw_argv)
        include_internal = bool(getattr(ns, "include_internal", False))
        payload = _commands_payload(include_internal=include_internal)
        if ns.cmd == "help":
            payload["schema_name"] = HELP
        if as_json:
            print(json.dumps(payload, sort_keys=True))
        else:
            names = [str(item["name"]) for item in payload["commands"] if isinstance(item, dict)]
            print("\n".join(names))
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
        from ..core.runtime.guards.env_guard import guard_no_network_mode

        guard_no_network_mode(True)
        restore_network = install_no_network_guard()

    try:
        should_emit_diag = bool(ctx.verbose) and (not ctx.quiet) and (ctx.output_format != "json")
        if should_emit_diag:
            print(f"run_id={ctx.run_id}", file=sys.stderr)
        if ctx.require_clean_git and ctx.git_dirty:
            raise ScriptError("git workspace is dirty; rerun without --require-clean-git or commit changes", ERR_CONFIG)
        if decision.allow_effective and should_emit_diag:
            print(
                f"NETWORK ENABLED: command={ns.cmd} group={decision.group} reason={decision.reason}",
                file=sys.stderr,
            )
        if should_emit_diag:
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
