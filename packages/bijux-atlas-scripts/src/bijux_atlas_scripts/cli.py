from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from .run_id import make_run_id
from .version import __version__


def _version_with_sha() -> str:
    base = f"bijux-atlas-scripts {__version__}"
    try:
        repo_root = Path(__file__).resolve().parents[4]
        sha = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], cwd=repo_root, text=True).strip()
        if sha:
            return f"{base}+{sha}"
    except Exception:
        pass
    return f"{base}+unknown"


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="bijux-atlas-scripts")
    p.add_argument("--version", action="version", version=_version_with_sha())
    sub = p.add_subparsers(dest="cmd")

    doctor = sub.add_parser("doctor", help="print basic runtime context")
    doctor.add_argument("--json", action="store_true")

    ns = p.parse_args(argv)
    if ns.cmd == "doctor":
        payload = {"tool": "bijux-atlas-scripts", "run_id": make_run_id(), "version": __version__}
        if ns.json:
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"tool={payload['tool']} version={payload['version']} run_id={payload['run_id']}")
        return 0

    p.print_help()
    return 0
