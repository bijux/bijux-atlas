#!/usr/bin/env python3
# owner: contracts
# purpose: public wrapper for canonical contracts script check_breaking_contract_change.py.
# stability: public
# called-by: make openapi-drift, make ci
from __future__ import annotations
import runpy
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[4]
target = root / "scripts" / "contracts" / "check_breaking_contract_change.py"
sys.argv[0] = str(target)
runpy.run_path(str(target), run_name="__main__")
