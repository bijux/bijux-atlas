#!/usr/bin/env python3
# Purpose: public compatibility wrapper for perf tooling script.
# Inputs: command-line args and env vars.
# Outputs: delegates execution to canonical ops/load/scripts implementation.
# Owner: performance
# Stability: public
from pathlib import Path
import runpy
import sys

ROOT = Path(__file__).resolve().parents[2]
TARGET = ROOT / "ops" / "load" / "scripts" / Path(__file__).name
sys.path.insert(0, str(TARGET.parent))
runpy.run_path(str(TARGET), run_name="__main__")
