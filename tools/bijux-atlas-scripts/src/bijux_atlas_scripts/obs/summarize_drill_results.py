#!/usr/bin/env python3
from __future__ import annotations

import runpy
from pathlib import Path

TARGET = (
    Path(__file__).resolve().parents[5]
    / "ops/obs/scripts/summarize_drill_results.py"
)

runpy.run_path(TARGET, run_name="__main__")
