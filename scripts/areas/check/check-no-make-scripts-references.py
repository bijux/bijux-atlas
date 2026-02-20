#!/usr/bin/env python3
# Purpose: enforce migration away from direct scripts/ references in make recipes.
# Inputs: Makefile + makefiles/*.mk and configs/layout/make-scripts-reference-exceptions.json.
# Outputs: non-zero on unapproved scripts/ references or expired exceptions.
from __future__ import annotations

import datetime as dt
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXCEPTIONS = ROOT / "configs/layout/make-scripts-reference-exceptions.json"
MAKEFILES = [ROOT / "Makefile", *sorted((ROOT / "makefiles").glob("*.mk"))]


def _load_exceptions() -> tuple[list[dict[str, str]], list[str]]:
    payload = json.loads(EXCEPTIONS.read_text(encoding="utf-8"))
    rows: list[dict[str, str]] = []
    errors: list[str] = []
    today = dt.date.today()
    for raw in payload.get("exceptions", []):
        if not isinstance(raw, dict):
            continue
        expiry = str(raw.get("expires_on", ""))
        rid = str(raw.get("id", "<missing-id>"))
        try:
            exp = dt.date.fromisoformat(expiry)
        except ValueError:
            errors.append(f"invalid expires_on for exception {rid}: `{expiry}`")
            continue
        if exp < today:
            errors.append(f"expired exception {rid}: {expiry}")
            continue
        rows.append(
            {
                "id": rid,
                "pattern": str(raw.get("pattern", "")),
                "owner": str(raw.get("owner", "")),
                "issue": str(raw.get("issue", "")),
            }
        )
    return rows, errors


def main() -> int:
    exceptions, errs = _load_exceptions()
    violations: list[str] = []

    for mk in MAKEFILES:
        for idx, line in enumerate(mk.read_text(encoding="utf-8").splitlines(), start=1):
            if "scripts/" not in line:
                continue
            if not line.startswith("\t"):
                continue
            if any(ex["pattern"] and ex["pattern"] in line for ex in exceptions):
                continue
            violations.append(f"{mk.relative_to(ROOT)}:{idx}: unapproved scripts/ reference in make recipe")

    if errs or violations:
        print("make scripts reference policy failed:", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        for v in violations[:200]:
            print(f"- {v}", file=sys.stderr)
        return 1
    print("make scripts reference policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
