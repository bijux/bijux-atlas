#!/usr/bin/env python3
# owner: contracts
# purpose: public wrapper for canonical contracts script check_sqlite_indexes_contract.py.
# stability: public
# called-by: make query-plan-gate, make ci-query-plan-gate
from __future__ import annotations
import runpy
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
target = root / "scripts" / "contracts" / "check_sqlite_indexes_contract.py"
sys.argv[0] = str(target)
runpy.run_path(str(target), run_name="__main__")
