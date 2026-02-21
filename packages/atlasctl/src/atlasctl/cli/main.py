from __future__ import annotations

import argparse
import importlib
import json
import os
import platform
import sys
from datetime import date
from pathlib import Path

from .. import __version__, layout, registry
from ..cli.registry import register_domain_parser, render_payload
from ..cli.registry import command_registry
from ..core.context import RunContext
from ..core.env import getenv, setdefault as env_setdefault, setenv
from ..core.env_guard import guard_no_network_mode
from ..core.exec import check_output
from ..core.fs import ensure_evidence_path
from ..core.logging import log_event
from ..core.serialize import dumps_json
from ..core.repo_root import try_find_repo_root
from ..errors import ScriptError
from ..exit_codes import ERR_CONFIG, ERR_INTERNAL
from ..network_guard import install_no_network_guard
from ..legacy.runner import run_legacy_script
from ..surface import run_surface
from .constants import CONFIGURE_HOOKS, DOMAINS, NO_NETWORK_FLAG_EXPIRY
from .output import build_base_payload, emit, no_network_flag_expired, render_error, resolve_output_format
DOMAIN_RUNNERS = {"registry": registry.run, "layout": layout.run}


def _import_attr(module_name: str, attr: str):
    return getattr(importlib.import_module(module_name), attr)


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="atlasctl")
    p.add_argument("--version", action="version", version=_version_string())
    p.add_argument("--json", action="store_true", help="emit JSON output")
    p.add_argument("--run-id", help="run identifier for artifacts")
    p.add_argument("--artifacts-dir", help="artifacts/evidence root path")
    p.add_argument("--evidence-root", help="evidence root path")
    p.add_argument("--run-dir", help="override run directory root")
    p.add_argument("--cwd", help="run command from an explicit repository root")
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
    for module_name, attr in CONFIGURE_HOOKS[:2]:
        _import_attr(module_name, attr)(sub)
    self_p = sub.add_parser("self-check", help="validate imports, config loading, and schema presence")
    self_p.add_argument("--json", action="store_true", help="emit JSON output")
    help_p = sub.add_parser("help", help="print command help")
    help_p.add_argument("--json", action="store_true", help="emit machine-readable command inventory")
    help_p.add_argument("--out-file", help="optional output path")
    explain_p = sub.add_parser("explain", help="describe command side-effects and external tools")
    explain_p.add_argument("command")
    explain_p.add_argument("--json", action="store_true", help="emit JSON output")

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
    config_p = sub.add_parser("config", help="configuration commands (alias over `configs`)")
    config_sub = config_p.add_subparsers(dest="config_cmd", required=True)
    config_dump = config_sub.add_parser("dump", help="dump canonical config payload")
    config_dump.add_argument("--report", choices=["text", "json"], default="json")
    config_validate = config_sub.add_parser("validate", help="validate config schemas and policy")
    config_validate.add_argument("--report", choices=["text", "json"], default="text")
    config_validate.add_argument("--emit-artifacts", action="store_true")
    config_drift = config_sub.add_parser("drift", help="check generated config drift")
    config_drift.add_argument("--report", choices=["text", "json"], default="text")

    for name in DOMAINS:
        register_domain_parser(sub, name, f"{name} domain commands")
    for module_name, attr in CONFIGURE_HOOKS[2:]:
        _import_attr(module_name, attr)(sub)

    doctor_p = sub.add_parser("doctor", help="show tooling and context diagnostics")
    doctor_p.add_argument("--json", action="store_true", help="emit JSON output")
    doctor_p.add_argument("--out-file", help="optional output path for JSON report")

    completion_p = sub.add_parser("completion", help="emit shell completion stub")
    completion_p.add_argument("shell", choices=["bash", "zsh", "fish"])
    completion_p.add_argument("--json", action="store_true", help="emit JSON output")

    clean_p = sub.add_parser("clean", help="clean scripts artifacts under approved roots only")
    clean_p.add_argument("--older-than-days", type=int)
    clean_p.add_argument("--json", action="store_true", help="emit JSON output")
    fix_p = sub.add_parser("fix", help="run explicit fixers (separate from checks)")
    fix_p.add_argument("thing", nargs="?", default="list", help="fixer id or `list`")
    fix_p.add_argument("--json", action="store_true", help="emit JSON output")

    return p


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
    if not out_file:
        return
    out_path = ensure_evidence_path(ctx, Path(out_file))
    out_path.write_text(payload + "\n", encoding="utf-8")


def _commands_payload() -> dict[str, object]:
    return {
        "schema_name": "atlasctl.commands.v1",
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "run_id": "",
        "commands": [
            {
                "name": c.name,
                "help": c.help_text,
                "stable": c.stable,
                "touches": list(c.touches),
                "tools": list(c.tools),
                "failure_modes": list(c.failure_modes),
                "owner": c.owner,
                "doc_link": c.doc_link,
            }
            for c in sorted(command_registry(), key=lambda c: c.name)
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
        "schema_name": "atlasctl.runtime_contracts.v1",
        "schema_version": 1,
        "tool": "atlasctl",
        "run_id": ctx.run_id,
        "status": "ok",
        "allowed_write_roots": [str(ctx.evidence_root), str(root), str(root / "reports"), str(root / "logs")],
        "forbidden_roots": [str(ctx.repo_root / p) for p in ("ops", "docs", "configs", "makefiles", "crates")],
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


def main(argv: list[str] | None = None) -> int:
    raw_argv = argv if argv is not None else sys.argv[1:]
    p = build_parser()
    ns = p.parse_args(argv)
    if ns.format and "--json" in raw_argv and ns.format != "json":
        raise ScriptError("conflicting output flags: use either --format json or --json", ERR_CONFIG)
    if ns.cwd:
        os.chdir(ns.cwd)
    if ns.no_network and no_network_flag_expired(date.today(), NO_NETWORK_FLAG_EXPIRY):
        print("--no-network flag expired; use --network=forbid", file=sys.stderr)
        return ERR_CONFIG
    evidence_root = ns.artifacts_dir or ns.evidence_root
    fmt = resolve_output_format(cli_json=("--json" in raw_argv), cli_format=ns.format, ci_present=bool(getenv("CI")))
    ctx = RunContext.from_args(
        ns.run_id,
        evidence_root,
        ns.profile,
        ns.no_network,
        fmt,
        ns.network,
        ns.run_dir,
        ns.verbose,
        ns.quiet,
        ns.require_clean_git,
    )
    _apply_python_env(ctx)
    _emit_runtime_contracts(ctx, ns.cmd, raw_argv)
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
            emit(
                {
                    **build_base_payload(ctx),
                    "atlasctl_version": __version__,
                    "scripts_version": _version_string().split()[1],
                },
                as_json,
            )
            return 0
        if ns.cmd == "env":
            return _import_attr("atlasctl.env.command", "run_env_command")(ctx, ns)
        if ns.cmd == "paths":
            return _import_attr("atlasctl.paths.command", "run_paths_command")(ctx, ns)
        if ns.cmd == "self-check":
            payload = build_base_payload(ctx)
            payload["checks"] = {
                "imports": "ok",
                "config_dir_exists": (ctx.repo_root / "configs").is_dir(),
                "schemas_dir_exists": (ctx.repo_root / "configs" / "_schemas").is_dir(),
                "contracts_schema_exists": (ctx.repo_root / "configs/contracts/atlasctl-output.schema.json").is_file(),
                "python_executable": sys.executable,
                "python_version": platform.python_version(),
            }
            payload["status"] = "ok" if payload["checks"]["config_dir_exists"] and payload["checks"]["schemas_dir_exists"] and payload["checks"]["contracts_schema_exists"] else "fail"
            emit(payload, as_json)
            return 0 if payload["status"] == "ok" else ERR_CONFIG
        if ns.cmd == "help":
            payload = _commands_payload()
            payload["schema_name"] = "atlasctl.help.v1"
            payload["run_id"] = ctx.run_id
            rendered = dumps_json(payload, pretty=not ns.json)
            if ns.out_file:
                _write_payload_if_requested(ctx, ns.out_file, rendered)
            print(rendered)
            return 0
        if ns.cmd == "explain":
            desc = _import_attr("atlasctl.commands.explain", "describe_command")(ns.command)
            payload = {
                "schema_name": "atlasctl.explain.v1",
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "run_id": ctx.run_id,
                "command": ns.command,
                **desc,
            }
            print(dumps_json(payload, pretty=not as_json))
            return 0
        if ns.cmd == "completion":
            payload = {"schema_version": 1, "tool": "atlasctl", "shell": ns.shell, "status": "ok"}
            if as_json:
                print(json.dumps(payload, sort_keys=True))
            else:
                print(f"# completion for {ns.shell} is not yet generated; use `atlasctl help --json`")
            return 0
        if ns.cmd == "clean":
            payload = _import_attr("atlasctl.env.command", "clean_scripts_artifacts")(ctx, ns.older_than_days)
            if as_json or ns.json:
                print(json.dumps(payload, sort_keys=True))
            else:
                print(f"removed={len(payload.get('removed', []))}")
            return 0
        if ns.cmd == "fix":
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "thing": ns.thing,
                "fixers": [],
                "note": "Fixers are explicit actions and are never run as part of `atlasctl check`.",
            }
            print(dumps_json(payload, pretty=not (as_json or ns.json)))
            return 0
        if ns.cmd == "run":
            if ns.dry_run:
                emit(
                    {
                        "schema_version": 1,
                        "tool": "atlasctl",
                        "status": "ok",
                        "script": ns.script,
                        "args": ns.args,
                    },
                    as_json,
                )
                return 0
            return run_legacy_script(ns.script, ns.args, ctx)
        if ns.cmd == "validate-output":
            return _import_attr("atlasctl.contracts.output", "validate_json_output")(ns.schema, ns.file, ns.json)
        if ns.cmd == "surface":
            return run_surface(ns.json, ns.out_file, ctx)
        if ns.cmd == "commands":
            payload = _commands_payload()
            payload["run_id"] = ctx.run_id
            rendered = dumps_json(payload, pretty=not ns.json)
            if ns.out_file:
                _write_payload_if_requested(ctx, ns.out_file, rendered)
            print(rendered)
            return 0
        if ns.cmd == "doctor":
            return _import_attr("atlasctl.commands.doctor", "run_doctor")(ctx, ns.json, ns.out_file)
        if ns.cmd == "docs":
            return _import_attr("atlasctl.commands.docs.legacy", "run_docs_command")(ctx, ns)
        if ns.cmd == "configs":
            return _import_attr("atlasctl.configs.command", "run_configs_command")(ctx, ns)
        if ns.cmd == "config":
            mapped = argparse.Namespace(**vars(ns))
            mapped.configs_cmd = {"dump": "print", "validate": "validate", "drift": "drift"}[ns.config_cmd]
            return _import_attr("atlasctl.configs.command", "run_configs_command")(ctx, mapped)
        if ns.cmd == "contracts":
            return _import_attr("atlasctl.contracts.command", "run_contracts_command")(ctx, ns)
        if ns.cmd == "docker":
            return _import_attr("atlasctl.docker.command", "run_docker_command")(ctx, ns)
        if ns.cmd == "ci":
            return _import_attr("atlasctl.ci.command", "run_ci_command")(ctx, ns)
        if ns.cmd == "check":
            return _import_attr("atlasctl.checks.command", "run_check_command")(ctx, ns)
        if ns.cmd == "deps":
            return _import_attr("atlasctl.deps.command", "run_deps_command")(ctx, ns)
        if ns.cmd == "gen":
            return _import_attr("atlasctl.gen.command", "run_gen_command")(ctx, ns)
        if ns.cmd == "policies":
            return _import_attr("atlasctl.policies.command", "run_policies_command")(ctx, ns)
        if ns.cmd == "make":
            return _import_attr("atlasctl.make.command", "run_make_command")(ctx, ns)
        if ns.cmd == "migration":
            return _import_attr("atlasctl.migrate.command", "run_migrate_command")(ctx, ns)
        if ns.cmd == "ops":
            return _import_attr("atlasctl.commands.ops.legacy", "run_ops_command")(ctx, ns)
        if ns.cmd == "inventory":
            return _import_attr("atlasctl.inventory.command", "run_inventory")(
                ctx, ns.category, ns.format, ns.out_dir, ns.dry_run, ns.check, ns.command
            )
        if ns.cmd == "report":
            return _import_attr("atlasctl.reporting.command", "run_report_command")(ctx, ns)
        if ns.cmd == "lint":
            return _import_attr("atlasctl.lint.command", "run_lint_command")(ctx, ns)
        if ns.cmd == "test":
            return _import_attr("atlasctl.test_tools.command", "run_test_command")(ctx, ns)
        if ns.cmd == "compat":
            return _import_attr("atlasctl.commands.compat", "run_compat_command")(ctx, ns)
        if ns.cmd == "legacy":
            return _import_attr("atlasctl.legacy.command", "run_legacy_command")(ctx, ns)
        if ns.cmd == "python":
            return _import_attr("atlasctl.python_tools.command", "run_python_command")(ctx, ns)
        if ns.cmd in {"ports", "artifacts", "k8s", "stack", "obs", "load", "e2e", "datasets", "cleanup", "scenario"}:
            return _import_attr("atlasctl.orchestrate.command", "run_orchestrate_command")(ctx, ns)
        if ns.cmd == "gates":
            return _import_attr("atlasctl.gates.command", "run_gates_command")(ctx, ns)
        if ns.cmd in DOMAINS:
            payload_obj = DOMAIN_RUNNERS[ns.cmd](ctx)
            payload = render_payload(payload_obj, as_json)
            _write_payload_if_requested(ctx, ns.out_file, payload)
            print(payload)
            return 0
        return 2
    except ScriptError as exc:
        print(render_error(as_json=(ctx.output_format == "json"), message=str(exc), code=exc.code), file=sys.stderr)
        return exc.code
    except Exception as exc:  # pragma: no cover
        as_json = "ctx" in locals() and ctx.output_format == "json"
        print(
            render_error(as_json=as_json, message=f"internal error: {exc}", code=ERR_INTERNAL),
            file=sys.stderr,
        )
        return ERR_INTERNAL
    finally:
        if restore_network:
            restore_network()


if __name__ == "__main__":
    raise SystemExit(main())
