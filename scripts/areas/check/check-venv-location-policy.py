#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ALLOWED_PREFIXES = (
    "artifacts/bijux-atlas-scripts/venv/.venv",
    "artifacts/isolate/py/scripts/.venv",
)


def main() -> int:
    proc = subprocess.run(["git", "ls-files", "--others", "--cached", "--exclude-standard"], cwd=ROOT, text=True, capture_output=True, check=False)
    paths = [p.strip() for p in proc.stdout.splitlines() if p.strip()]
    violations: list[str] = []
    for rel in paths:
        parts = Path(rel).parts
        if ".venv" not in parts:
            continue
        if any(rel.startswith(prefix) for prefix in ALLOWED_PREFIXES):
            continue
        violations.append(rel)

    root_venv = ROOT / ".venv"
    if root_venv.exists() and not any(str(root_venv).startswith(str(ROOT / p)) for p in ALLOWED_PREFIXES):
        violations.append(".venv")

    if violations:
        print("venv location policy failed:", file=sys.stderr)
        for item in violations[:200]:
            print(f"- forbidden .venv location: {item}", file=sys.stderr)
        print("allowed .venv roots:", file=sys.stderr)
        for p in ALLOWED_PREFIXES:
            print(f"- {p}", file=sys.stderr)
        return 1

    print("venv location policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
