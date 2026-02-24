#!/usr/bin/env python3
# Purpose: enforce that GitHub workflow run steps invoke make targets only.
# Inputs: .github/workflows/*.yml
# Outputs: non-zero exit when raw run commands bypass make.
from __future__ import annotations

import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists() and (base / ".github").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
WORKFLOWS = sorted((ROOT / ".github" / "workflows").glob("*.yml"))

run_line = re.compile(r"^\s*-\s*run:\s*(.+)\s*$")
allowed_prefixes = (
    "make ",
    "make\t",
)

violations: list[str] = []
for wf in WORKFLOWS:
    for idx, line in enumerate(wf.read_text(encoding="utf-8").splitlines(), start=1):
        m = run_line.match(line)
        if not m:
            continue
        cmd = m.group(1).strip()
        if cmd.startswith("|"):
            violations.append(f"{wf.relative_to(ROOT)}:{idx}: multiline run blocks are forbidden; use make target")
            continue
        if cmd.startswith("\"") and cmd.endswith("\""):
            cmd = cmd[1:-1].strip()
        if cmd.startswith(allowed_prefixes):
            continue
        violations.append(f"{wf.relative_to(ROOT)}:{idx}: run step must invoke make target, found `{cmd}`")

if violations:
    print("workflow make-only check failed:", file=sys.stderr)
    for v in violations:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("workflow make-only check passed")
