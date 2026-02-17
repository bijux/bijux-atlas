#!/usr/bin/env python3
# Purpose: enforce pinned ops tool versions from canonical ops/tool-versions.json.
# Inputs: ops/tool-versions.json (or configs/ops/tool-versions.json fallback) and local tool CLI outputs.
# Outputs: non-zero exit if installed version does not match pinned lockfile version.
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LOCK_CANDIDATES = [
    ROOT / "ops" / "tool-versions.json",
    ROOT / "configs" / "ops" / "tool-versions.json",
]

CMDS = {
    "kind": ["kind", "version"],
    "k6": ["k6", "version"],
    "helm": ["helm", "version", "--short"],
    "kubectl": ["kubectl", "version", "--client", "--output=yaml"],
}


def detect_version(tool: str) -> str:
    cmd = CMDS[tool]
    try:
        out = subprocess.check_output(cmd, cwd=ROOT, text=True, stderr=subprocess.STDOUT)
    except FileNotFoundError:
        raise RuntimeError(f"{tool} is not installed")
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(f"{tool} command failed: {exc.output.strip()}")

    match = re.search(r"v\d+\.\d+\.\d+", out)
    if not match:
        raise RuntimeError(f"could not parse {tool} version from output: {out.strip()}")
    return match.group(0)


def main() -> int:
    lock_path = next((path for path in LOCK_CANDIDATES if path.exists()), None)
    if lock_path is None:
        joined = ", ".join(str(path) for path in LOCK_CANDIDATES)
        print(f"missing lockfile (checked: {joined})", file=sys.stderr)
        return 1
    lock = json.loads(lock_path.read_text())
    tools = sys.argv[1:] or sorted(CMDS.keys())
    failed = False
    for tool in tools:
        if tool not in CMDS:
            print(f"unknown tool: {tool}", file=sys.stderr)
            failed = True
            continue
        expected = lock.get(tool)
        if not expected:
            print(f"missing pinned version for {tool} in {lock_path}", file=sys.stderr)
            failed = True
            continue
        try:
            actual = detect_version(tool)
        except RuntimeError as err:
            print(str(err), file=sys.stderr)
            failed = True
            continue
        if actual != expected:
            print(f"{tool} version mismatch: expected {expected}, got {actual}", file=sys.stderr)
            failed = True
        else:
            print(f"{tool} version ok: {actual}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
