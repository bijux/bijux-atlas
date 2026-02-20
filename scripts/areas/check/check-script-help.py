#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TARGETS = [
    ROOT / "scripts/bin/bijux-atlas-dev",
    ROOT / "scripts/areas/check/no-duplicate-script-names.sh",
    ROOT / "scripts/areas/check/no-direct-path-usage.sh",
    ROOT / "scripts/areas/ci/scripts-ci.sh",
]

errors: list[str] = []
for p in TARGETS:
    if not p.exists():
        errors.append(f"missing help-gated script: {p.relative_to(ROOT)}")
        continue
    proc = subprocess.run([str(p), "--help"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if proc.returncode != 0:
        errors.append(f"{p.relative_to(ROOT)}: --help exited {proc.returncode}")
        continue
    out = proc.stdout.lower()
    if "usage" not in out and "purpose" not in out and "contract" not in out:
        errors.append(f"{p.relative_to(ROOT)}: --help output missing usage/contract text")

if errors:
    print("script help contract failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("script help contract passed")
