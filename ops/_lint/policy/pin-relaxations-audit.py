#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REGISTRY = ROOT / "configs/policy/pin-relaxations.json"


def main() -> int:
    payload = json.loads(REGISTRY.read_text(encoding="utf-8"))
    today = dt.date.today()
    errors: list[str] = []
    active: list[dict[str, str]] = []
    expired: list[dict[str, str]] = []
    for entry in payload.get("exceptions", []):
        entry_id = str(entry.get("id", "")).strip()
        for key in ("id", "policy", "scope", "owner", "issue", "expiry", "justification"):
            if not str(entry.get(key, "")).strip():
                errors.append(f"{entry_id or '<missing-id>'}: missing required field `{key}`")
        expiry_raw = str(entry.get("expiry", ""))
        try:
            expiry = dt.date.fromisoformat(expiry_raw)
        except ValueError:
            errors.append(f"{entry_id or '<missing-id>'}: invalid expiry `{expiry_raw}`")
            continue
        if expiry < today:
            expired.append({"id": entry_id, "policy": str(entry.get("policy", "")), "expiry": expiry_raw})
            errors.append(f"{entry_id}: expired on {expiry_raw}")
        else:
            active.append({"id": entry_id, "policy": str(entry.get("policy", "")), "expiry": expiry_raw})

    out = ROOT / "ops/_artifacts/policy/pin-relaxations-audit.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "date": today.isoformat(),
                "active_relaxations": active,
                "expired_relaxations": expired,
                "errors": errors,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )

    if errors:
        for err in errors:
            print(f"pin-relaxations violation: {err}", file=sys.stderr)
        return 1
    print("pin relaxations audit passed")
    print(out.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
