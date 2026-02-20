#!/usr/bin/env python3
"""DIR_BUDGET_SHIM"""
import runpy
from pathlib import Path
runpy.run_path(Path(__file__).resolve().parent / "layout/no-k8s-test-fixups.py", run_name="__main__")
