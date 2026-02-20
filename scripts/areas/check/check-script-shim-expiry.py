#!/usr/bin/env python3
from __future__ import annotations

import json
from datetime import date
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
CFG = ROOT / "configs/layout/script-shim-expiries.json"


def main() -> int:
    data = json.loads(CFG.read_text(encoding="utf-8"))
    shims = data.get("shims", [])
    known = {entry["path"] for entry in shims if isinstance(entry, dict) and "path" in entry}
    errors: list[str] = []

    for path in sorted((ROOT / "scripts/bin").glob("*")):
        if not path.is_file() or path.name == "bijux-atlas-scripts":
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "DEPRECATED:" not in text:
            continue
        rel = path.relative_to(ROOT).as_posix()
        if rel not in known:
            errors.append(f"shim missing expiry metadata: {rel}")

    today = date.today()
    for row in shims:
        rel = row.get("path", "")
        if not rel:
            errors.append("shim metadata missing path")
            continue
        p = ROOT / rel
        if not p.exists():
            errors.append(f"shim metadata points to missing file: {rel}")
            continue
        exp = date.fromisoformat(str(row.get("expires_on", "")))
        if exp < today:
            errors.append(f"shim expired: {rel} expired_on={exp.isoformat()}")

    if errors:
        print("script shim expiry check failed", file=sys.stderr)
        for item in errors:
            print(f"- {item}", file=sys.stderr)
        return 1

    print("script shim expiry check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
