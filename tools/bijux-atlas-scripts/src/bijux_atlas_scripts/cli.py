from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from . import contracts, layout, registry
from .configs.command import configure_configs_parser, run_configs_command
from .core.context import RunContext
from .core.fs import ensure_evidence_path
from .core.logging import log_event
from .docs.command import configure_docs_parser, run_docs_command
from .doctor import run_doctor
from .domain_cmd import register_domain_parser, render_payload
from .errors import ScriptError
from .exit_codes import ERR_INTERNAL
from .inventory.command import configure_inventory_parser, run_inventory
from .make.command import configure_make_parser, run_make_command
from .network_guard import install_no_network_guard
from .ops.command import configure_ops_parser, run_ops_command
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
    p = argparse.ArgumentParser(prog="bijux-atlas-scripts")
    p.add_argument("--version", action="version", version=_version_string())
    p.add_argument("--run-id", help="run identifier for artifacts")
    p.add_argument("--evidence-root", help="evidence root path")
    p.add_argument("--profile", help="profile id")
    p.add_argument("--no-network", action="store_true", help="deny outbound network calls")
    sub = p.add_subparsers(dest="cmd", required=True)

    run_p = sub.add_parser("run", help="run an internal python script by repo-relative path")
    run_p.add_argument("script")
    run_p.add_argument("args", nargs=argparse.REMAINDER)

    val_p = sub.add_parser("validate-output", help="validate JSON output against schema")
    val_p.add_argument("--schema", required=True)
    val_p.add_argument("--file", required=True)
    val_p.add_argument("--json", action="store_true", help="emit JSON status output")

    surface_p = sub.add_parser("surface", help="print scripts command ownership surface")
    surface_p.add_argument("--json", action="store_true", help="emit JSON output")
    surface_p.add_argument("--out-file", help="optional output path for JSON report")

    domain_names = ("contracts", "registry", "layout")
    for name in domain_names:
        register_domain_parser(sub, name, f"{name} domain commands")
    configure_configs_parser(sub)
    configure_policies_parser(sub)
    configure_docs_parser(sub)
    configure_make_parser(sub)
    configure_ops_parser(sub)
    configure_inventory_parser(sub)
    configure_report_parser(sub)

    doctor_p = sub.add_parser("doctor", help="show tooling and context diagnostics")
    doctor_p.add_argument("--json", action="store_true", help="emit JSON output")
    doctor_p.add_argument("--out-file", help="optional output path for JSON report")

    return p


def _version_string() -> str:
    base = "bijux-atlas-scripts 0.1.0"
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


def main(argv: list[str] | None = None) -> int:
    p = build_parser()
    ns = p.parse_args(argv)
    ctx = RunContext.from_args(ns.run_id, ns.evidence_root, ns.profile, ns.no_network)
    restore_network = None
    if ctx.no_network:
        restore_network = install_no_network_guard()
    try:
        log_event(ctx, "info", "cli", "start", cmd=ns.cmd)
        if ns.cmd == "run":
            return run_legacy_script(ns.script, ns.args, ctx)
        if ns.cmd == "validate-output":
            return validate_json_output(ns.schema, ns.file, ns.json)
        if ns.cmd == "surface":
            return run_surface(ns.json, ns.out_file)
        if ns.cmd == "doctor":
            return run_doctor(ctx, ns.json, ns.out_file)
        if ns.cmd == "docs":
            return run_docs_command(ctx, ns)
        if ns.cmd == "configs":
            return run_configs_command(ctx, ns)
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
        if ns.cmd in DOMAINS:
            payload_obj = DOMAINS[ns.cmd](ctx)
            payload = render_payload(payload_obj, bool(ns.json))
            _write_payload_if_requested(ctx, ns.out_file, payload)
            print(payload)
            return 0
        return 2
    except ScriptError as exc:
        print(str(exc), file=sys.stderr)
        return exc.code
    except Exception as exc:  # pragma: no cover
        print(f"internal error: {exc}", file=sys.stderr)
        return ERR_INTERNAL
    finally:
        if restore_network:
            restore_network()


if __name__ == "__main__":
    raise SystemExit(main())
