from __future__ import annotations

import argparse

from .output_contract import validate_json_output
from .runner import run_legacy_script


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="bijux-atlas-scripts")
    sub = p.add_subparsers(dest="cmd", required=True)

    run_p = sub.add_parser("run", help="run an internal python script by repo-relative path")
    run_p.add_argument("script")
    run_p.add_argument("args", nargs=argparse.REMAINDER)

    val_p = sub.add_parser("validate-output", help="validate JSON output against schema")
    val_p.add_argument("--schema", required=True)
    val_p.add_argument("--file", required=True)

    return p


def main(argv: list[str] | None = None) -> int:
    p = build_parser()
    ns = p.parse_args(argv)
    if ns.cmd == "run":
        return run_legacy_script(ns.script, ns.args)
    if ns.cmd == "validate-output":
        return validate_json_output(ns.schema, ns.file)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
