from __future__ import annotations

import argparse
import sys
from pathlib import Path

from . import configs, docs, inventory, make, ops, policies, report
from .doctor import run_doctor
from .domain_cmd import register_domain_parser, render_payload
from .errors import ScriptError
from .evidence_policy import ensure_evidence_path
from .exit_codes import ERR_INTERNAL
from .network_guard import install_no_network_guard
from .output_contract import validate_json_output
from .run_context import RunContext
from .runner import run_legacy_script
from .structured_log import log_event
from .surface import run_surface

DOMAINS = {
    "ops": ops.run,
    "docs": docs.run,
    "configs": configs.run,
    "policies": policies.run,
    "make": make.run,
    "inventory": inventory.run,
    "report": report.run,
}


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="bijux-atlas-scripts")
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

    for name in ("ops", "docs", "configs", "policies", "make", "inventory", "report"):
        register_domain_parser(sub, name, f"{name} domain commands")

    doctor_p = sub.add_parser("doctor", help="show tooling and context diagnostics")
    doctor_p.add_argument("--json", action="store_true", help="emit JSON output")
    doctor_p.add_argument("--out-file", help="optional output path for JSON report")

    return p


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
