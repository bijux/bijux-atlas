#!/usr/bin/env python3
# Purpose: enforce pinned ops tool versions from canonical configs/ops/tool-versions.json.
# Inputs: configs/ops/tool-versions.json and local tool CLI outputs.
# Outputs: non-zero exit if installed version does not match pinned lockfile version.
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
LOCK_PATH = ROOT / "configs" / "ops" / "tool-versions.json"

CMDS = {
    "kind": ["kind", "version"],
    "k6": ["k6", "version"],
    "helm": ["helm", "version", "--short"],
    "kubectl": ["kubectl", "version", "--client", "--output=yaml"],
    "jq": ["jq", "--version"],
    "yq": ["yq", "--version"],
    "python3": ["python3", "--version"],
}


def detect_version(tool: str) -> str:
    cmd = CMDS[tool]
    try:
        out = subprocess.check_output(cmd, cwd=ROOT, text=True, stderr=subprocess.STDOUT)
    except FileNotFoundError:
        raise RuntimeError(f"{tool} is not installed")
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(f"{tool} command failed: {exc.output.strip()}")

    match = re.search(r"(?:v|jq-)?(\d+\.\d+\.\d+)", out)
    if not match:
        raise RuntimeError(f"could not parse {tool} version from output: {out.strip()}")
    return f"v{match.group(1)}"


def normalize_version(version: str) -> str:
    return version[1:] if version.startswith("v") else version


def main() -> int:
    if not LOCK_PATH.exists():
        print(f"missing lockfile: {LOCK_PATH}", file=sys.stderr)
        return 1
    lock = json.loads(LOCK_PATH.read_text())
    tools = sys.argv[1:] or sorted(CMDS.keys())
    failed = False
    for tool in tools:
        if tool not in CMDS:
            print(f"unknown tool: {tool}", file=sys.stderr)
            failed = True
            continue
        expected = lock.get(tool)
        if tool == "python3" and expected is None:
            expected = lock.get("python")
        if not expected:
            print(f"missing pinned version for {tool} in {LOCK_PATH}", file=sys.stderr)
            failed = True
            continue
        try:
            actual = detect_version(tool)
        except RuntimeError as err:
            print(str(err), file=sys.stderr)
            failed = True
            continue
        if normalize_version(actual) != normalize_version(str(expected)):
            print(f"{tool} version mismatch: expected {expected}, got {actual}", file=sys.stderr)
            failed = True
        else:
            print(f"{tool} version ok: {actual}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
