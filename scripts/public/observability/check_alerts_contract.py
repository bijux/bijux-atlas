#!/usr/bin/env python3
# owner: operations
# purpose: public wrapper for canonical ops observability contract script check_alerts_contract.py.
# stability: public
# called-by: make ops-* targets
# Purpose: preserve stable public entrypoint while delegating to ops/observability/scripts/contracts.
# Inputs: argv passed through unchanged.
# Outputs: same as ops/observability/scripts/contracts/check_alerts_contract.py.
from __future__ import annotations
import runpy
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
target = root / "ops" / "observability" / "scripts" / "contracts" / "check_alerts_contract.py"
sys.argv[0] = str(target)
runpy.run_path(str(target), run_name="__main__")
