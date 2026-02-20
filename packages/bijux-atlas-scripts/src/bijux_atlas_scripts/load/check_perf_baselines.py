#!/usr/bin/env python3
from __future__ import annotations

import runpy
from pathlib import Path

TARGET = (
    Path(__file__).resolve().parents[5]
    / "ops/load/scripts/check_perf_baselines.py"
)

runpy.run_path(TARGET, run_name="__main__")
