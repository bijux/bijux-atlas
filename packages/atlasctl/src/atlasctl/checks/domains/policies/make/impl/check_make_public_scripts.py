#!/usr/bin/env python3
# Purpose: ensure make recipes call only scripts declared as public entrypoints.
# Inputs: Makefile + makefiles/*.mk and configs/ops/public-surface.json patterns.
# Outputs: non-zero exit when make calls non-public scripts.
from __future__ import annotations

import fnmatch
import json
import re
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
surface = json.loads((ROOT / "configs/ops/public-surface.json").read_text(encoding="utf-8"))
patterns = [f"{cmd}" for cmd in surface.get("ops_run_commands", []) if cmd.startswith("scripts/")]

mk_files = [ROOT / "Makefile"] + sorted((ROOT / "makefiles").glob("*.mk"))
text = "\n".join(p.read_text() for p in mk_files)
called = set(re.findall(r"\./(scripts/[^\s\"\\]+)", text))
called.update(re.findall(r"(?:python3|python)\s+(scripts/[^\s\"\\]+)", text))
called = sorted(path.rstrip(";") for path in called)

violations: list[str] = []
for path in called:
    if not any(fnmatch.fnmatch(path, pat) for pat in patterns):
        violations.append(path)

if violations:
    print("make references non-public scripts:")
    for v in violations:
        print(f"- {v}")
    raise SystemExit(1)
print("make public script gate passed")
