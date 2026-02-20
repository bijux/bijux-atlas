from __future__ import annotations

import argparse
import sys

from .doctor import run_doctor
from .errors import ScriptError
from .exit_codes import ERR_INTERNAL
from .network_guard import install_no_network_guard
from .output_contract import validate_json_output
from .run_context import RunContext
from .runner import run_legacy_script
from .structured_log import log_event
from .surface import run_surface


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

    doctor_p = sub.add_parser("doctor", help="show tooling and context diagnostics")
    doctor_p.add_argument("--json", action="store_true", help="emit JSON output")
    doctor_p.add_argument("--out-file", help="optional output path for JSON report")

    return p


def main(argv: list[str] | None = None) -> int:
    p = build_parser()
    ns = p.parse_args(argv)
    ctx = RunContext.from_args(ns.run_id, ns.evidence_root, ns.profile, ns.no_network)
    restore_network = None
    if ctx.no_network:
        restore_network = install_no_network_guard()
    try:
        log_event(ctx, "info", "start", cmd=ns.cmd)
        if ns.cmd == "run":
            return run_legacy_script(ns.script, ns.args, ctx)
        if ns.cmd == "validate-output":
            return validate_json_output(ns.schema, ns.file, ns.json)
        if ns.cmd == "surface":
            return run_surface(ns.json, ns.out_file)
        if ns.cmd == "doctor":
            return run_doctor(ctx, ns.json, ns.out_file)
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
