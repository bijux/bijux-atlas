from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys

from ..core.context import RunContext


def _tool_status(name: str, probe: list[str]) -> dict[str, str]:
    path = shutil.which(name)
    if not path:
        return {"name": name, "status": "missing", "path": "", "version": ""}
    try:
        proc = subprocess.run(probe, text=True, capture_output=True, check=False)
        line = (proc.stdout or proc.stderr).strip().splitlines()
        version = line[0] if line else ""
        status = "ok" if proc.returncode == 0 else "error"
    except Exception:
        version = ""
        status = "error"
    return {"name": name, "status": status, "path": path, "version": version}


def run_install_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.install_cmd != "doctor":
        return 2
    tools = [
        _tool_status("python3", ["python3", "--version"]),
        _tool_status("git", ["git", "--version"]),
        _tool_status("make", ["make", "--version"]),
    ]
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if all(row["status"] == "ok" for row in tools) else "error",
        "run_id": ctx.run_id,
        "python": sys.version.split()[0],
        "repo_root": str(ctx.repo_root),
        "tools": tools,
    }
    if ns.json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"install doctor: {payload['status']}")
        for row in tools:
            version = row["version"] or "n/a"
            print(f"- {row['name']}: {row['status']} ({version})")
    return 0 if payload["status"] == "ok" else 1


def configure_install_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("install", help="installation checks and doctor commands")
    p_sub = p.add_subparsers(dest="install_cmd", required=True)
    doctor = p_sub.add_parser("doctor", help="validate python environment and required external tools")
    doctor.add_argument("--json", action="store_true", help="emit JSON output")
