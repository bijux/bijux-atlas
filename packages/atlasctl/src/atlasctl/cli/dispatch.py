from __future__ import annotations

import argparse
import json
import platform
import sys
from collections.abc import Callable

from .. import __version__
from ..cli.registry import command_spec, render_payload
from ..core.context import RunContext
from ..core.script_runner import run_script
from ..core.serialize import dumps_json
from ..contracts.ids import EXPLAIN, HELP
from ..contracts.validate_self import validate_self
from ..errors import ScriptError
from ..exit_codes import ERR_CONFIG
from ..surface import run_surface
from .constants import DOMAINS
from .output import build_base_payload, emit


def dispatch_command(
    ctx: RunContext,
    ns: argparse.Namespace,
    as_json: bool,
    import_attr: Callable[[str, str], object],
    commands_payload: Callable[[], dict[str, object]],
    write_payload: Callable[[RunContext, str | None, str], None],
    domain_runners: dict[str, Callable[[RunContext], object]],
    version_string: Callable[[], str],
) -> int:
    def _resolve_explain_name() -> str:
        first = getattr(ns, "subject_or_name", "")
        second = getattr(ns, "name", None)
        if first == "command":
            if not second:
                raise ScriptError("usage: atlasctl explain command <name>", ERR_CONFIG)
            return second
        if second is not None:
            raise ScriptError("usage: atlasctl explain command <name>", ERR_CONFIG)
        return first

    if getattr(ns, "dry_run", False):
        spec = command_spec(ns.cmd)
        if spec is not None and spec.effect_level == "effectful":
            emit(
                {
                    **build_base_payload(ctx),
                    "status": "ok",
                    "dry_run": True,
                    "command": ns.cmd,
                    "effect_level": spec.effect_level,
                    "run_id_mode": spec.run_id_mode,
                },
                as_json,
            )
            return 0
    if ns.cmd == "version":
        emit({**build_base_payload(ctx), "atlasctl_version": __version__, "scripts_version": version_string().split()[1]}, as_json)
        return 0
    if ns.cmd == "env":
        return import_attr("atlasctl.env.command", "run_env_command")(ctx, ns)
    if ns.cmd == "paths":
        return import_attr("atlasctl.paths.command", "run_paths_command")(ctx, ns)
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
        payload["status"] = "ok" if all((payload["checks"]["config_dir_exists"], payload["checks"]["schemas_dir_exists"], payload["checks"]["contracts_schema_exists"])) else "fail"
        emit(payload, as_json)
        return 0 if payload["status"] == "ok" else ERR_CONFIG
    if ns.cmd == "help":
        payload = commands_payload(include_internal=bool(getattr(ns, "include_internal", False)))
        payload["schema_name"] = HELP
        payload["run_id"] = ctx.run_id
        validate_self(HELP, payload)
        rendered = dumps_json(payload, pretty=not ns.json)
        write_payload(ctx, ns.out_file, rendered)
        print(rendered)
        return 0
    if ns.cmd == "explain":
        explain_name = _resolve_explain_name()
        desc = import_attr("atlasctl.commands.explain", "describe_command")(explain_name)
        payload = {
            "schema_name": EXPLAIN,
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "run_id": ctx.run_id,
            "command": explain_name,
            **desc,
        }
        validate_self(EXPLAIN, payload)
        print(dumps_json(payload, pretty=not as_json))
        return 0
    if ns.cmd == "completion":
        payload = {"schema_version": 1, "tool": "atlasctl", "shell": ns.shell, "status": "ok"}
        print(json.dumps(payload, sort_keys=True) if as_json else f"# completion for {ns.shell} is not yet generated; use `atlasctl help --json`")
        return 0
    if ns.cmd == "clean":
        payload = import_attr("atlasctl.env.command", "clean_scripts_artifacts")(ctx, ns.older_than_days)
        print(json.dumps(payload, sort_keys=True) if (as_json or ns.json) else f"removed={len(payload.get('removed', []))}")
        return 0
    if ns.cmd == "fix":
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "thing": ns.thing, "fixers": [], "note": "Fixers are explicit actions and are never run as part of `atlasctl check`."}
        print(dumps_json(payload, pretty=not (as_json or ns.json)))
        return 0
    if ns.cmd == "run":
        if ns.script == "suite":
            if not ns.args:
                raise ScriptError("usage: atlasctl run suite <name> [--list|--json|--target-dir PATH]", ERR_CONFIG)
            suite_name = ns.args[0]
            list_flag = "--list" in ns.args[1:]
            json_flag = "--json" in ns.args[1:]
            target_dir: str | None = None
            if "--target-dir" in ns.args[1:]:
                idx = ns.args.index("--target-dir")
                if idx + 1 >= len(ns.args):
                    raise ScriptError("missing value for --target-dir", ERR_CONFIG)
                target_dir = ns.args[idx + 1]
            mapped = argparse.Namespace(suite_cmd=suite_name, json=json_flag, list=list_flag, target_dir=target_dir)
            return import_attr("atlasctl.suite.command", "run_suite_command")(ctx, mapped)
        if ns.dry_run:
            emit({"schema_version": 1, "tool": "atlasctl", "status": "ok", "script": ns.script, "args": ns.args}, as_json)
            return 0
        return run_script(ns.script, ns.args, ctx)
    if ns.cmd == "validate-output":
        return import_attr("atlasctl.contracts.output", "validate_json_output")(ns.schema, ns.file, ns.json)
    if ns.cmd == "surface":
        return run_surface(ns.json, ns.out_file, ctx)
    if ns.cmd == "commands":
        if ns.commands_cmd == "lint":
            lint = import_attr("atlasctl.checks.repo.contracts.command_contracts", "command_lint_payload")
            payload = lint(ctx.repo_root)
            code = 0 if payload["status"] == "ok" else ERR_CONFIG
            print(dumps_json(payload, pretty=not ns.json))
            return code
        include_internal = bool(getattr(ns, "include_internal", False) or ns.commands_cmd == "compat-check")
        payload = commands_payload(include_internal=include_internal)
        payload["run_id"] = ctx.run_id
        if getattr(ns, "verify_stability", False) or ns.commands_cmd == "compat-check":
            golden_path = ctx.repo_root / "packages/atlasctl/tests/goldens/commands.json.golden"
            if not golden_path.exists():
                raise ScriptError(f"missing commands stability golden: {golden_path.relative_to(ctx.repo_root)}", ERR_CONFIG)
            expected = json.loads(golden_path.read_text(encoding="utf-8"))
            compare_current = dict(payload)
            compare_expected = dict(expected)
            compare_current["run_id"] = ""
            compare_expected["run_id"] = ""
            if compare_current != compare_expected:
                raise ScriptError("commands stability verification failed against commands.json.golden", ERR_CONFIG)
        rendered = dumps_json(payload, pretty=not ns.json)
        write_payload(ctx, ns.out_file, rendered)
        print(rendered)
        return 0
    if ns.cmd == "doctor":
        return import_attr("atlasctl.commands.doctor", "run_doctor")(ctx, ns.json, getattr(ns, "out_file", None))
    if ns.cmd == "config":
        mapped = argparse.Namespace(**vars(ns))
        mapped.configs_cmd = {"dump": "print", "validate": "validate", "drift": "drift"}[ns.config_cmd]
        return import_attr("atlasctl.configs.command", "run_configs_command")(ctx, mapped)
    if ns.cmd == "inventory":
        return import_attr("atlasctl.inventory.command", "run_inventory")(ctx, ns.category, ns.format, ns.out_dir, ns.dry_run, ns.check, ns.command)

    module_dispatch = {
        "docs": ("atlasctl.commands.docs.runtime", "run_docs_command"),
        "configs": ("atlasctl.configs.command", "run_configs_command"),
        "contracts": ("atlasctl.contracts.command", "run_contracts_command"),
        "docker": ("atlasctl.docker.command", "run_docker_command"),
        "ci": ("atlasctl.ci.command", "run_ci_command"),
        "check": ("atlasctl.checks.command", "run_check_command"),
        "list": ("atlasctl.commands.listing", "run_list_command"),
        "deps": ("atlasctl.deps.command", "run_deps_command"),
        "gen": ("atlasctl.gen.command", "run_gen_command"),
        "policies": ("atlasctl.policies.command", "run_policies_command"),
        "repo": ("atlasctl.repo.command", "run_repo_command"),
        "make": ("atlasctl.make.command", "run_make_command"),
        "migration": ("atlasctl.migrate.command", "run_migrate_command"),
        "ops": ("atlasctl.commands.ops.runtime", "run_ops_command"),
        "report": ("atlasctl.reporting.command", "run_report_command"),
        "lint": ("atlasctl.lint.command", "run_lint_command"),
        "test": ("atlasctl.test_tools.command", "run_test_command"),
        "suite": ("atlasctl.suite.command", "run_suite_command"),
        "compat": ("atlasctl.commands.compat", "run_compat_command"),
        "legacy": ("atlasctl.commands.legacy_inventory", "run_legacy_command"),
        "python": ("atlasctl.python_tools.command", "run_python_command"),
        "gates": ("atlasctl.gates.command", "run_gates_command"),
    }
    if ns.cmd in module_dispatch:
        module_name, attr = module_dispatch[ns.cmd]
        return import_attr(module_name, attr)(ctx, ns)
    if ns.cmd in {"ports", "artifacts", "k8s", "stack", "obs", "load", "e2e", "datasets", "cleanup", "scenario"}:
        return import_attr("atlasctl.orchestrate.command", "run_orchestrate_command")(ctx, ns)
    if ns.cmd in DOMAINS:
        payload_obj = domain_runners[ns.cmd](ctx)
        payload = render_payload(payload_obj, as_json)
        write_payload(ctx, ns.out_file, payload)
        print(payload)
        return 0
    raise ScriptError(f"unknown command: {ns.cmd}", ERR_CONFIG)
