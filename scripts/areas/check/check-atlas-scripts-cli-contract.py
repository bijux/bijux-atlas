#!/usr/bin/env python3
# Purpose: verify atlas-scripts CLI deterministic help and version contract.
# Inputs: scripts/bin/bijux-atlas-scripts and VERSION.
# Outputs: non-zero on nondeterministic help or version mismatch.
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CLI = ROOT / "scripts" / "bin" / "bijux-atlas-scripts"
PYPROJECT = ROOT / "tools" / "bijux-atlas-scripts" / "pyproject.toml"


def _run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CLI), *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def main() -> int:
    errs: list[str] = []
    expected_version = ""
    for ln in PYPROJECT.read_text(encoding="utf-8").splitlines():
        stripped = ln.strip()
        if stripped.startswith("version = "):
            expected_version = stripped.split("=", 1)[1].strip().strip('"').strip("'")
            break

    h1 = _run(["--help"])
    h2 = _run(["--help"])
    if h1.returncode != 0 or h2.returncode != 0:
        errs.append("atlas-scripts --help must exit 0")
    if h1.stdout != h2.stdout:
        errs.append("atlas-scripts --help output is not deterministic")

    v = _run(["--version"])
    if v.returncode != 0:
        errs.append("atlas-scripts --version must exit 0")
    else:
        out = (v.stdout or v.stderr).strip()
        if expected_version and expected_version not in out:
            errs.append(f"atlas-scripts version mismatch: expected {expected_version}, got `{out}`")

    if errs:
        print("atlas-scripts cli contract check failed:", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("atlas-scripts cli contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
