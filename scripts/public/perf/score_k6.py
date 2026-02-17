#!/usr/bin/env python3
# owner: operations
# purpose: public wrapper for canonical ops load script score_k6.py.
# stability: public
# called-by: make ops-* targets
# Purpose: preserve stable public entrypoint while delegating to ops/load/scripts.
# Inputs: argv passed through unchanged.
# Outputs: same as ops/load/scripts/score_k6.py.
from __future__ import annotations
import runpy
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
target = root / "ops" / "load" / "scripts" / "score_k6.py"
sys.argv[0] = str(target)
runpy.run_path(str(target), run_name="__main__")
