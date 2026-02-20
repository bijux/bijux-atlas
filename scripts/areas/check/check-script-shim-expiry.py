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
    max_active = int(data.get("max_active_shims", 9999))
    shim_paths = []
    for base in (ROOT / "scripts/bin", ROOT / "bin"):
        if not base.exists():
            continue
        for path in sorted(base.glob("*")):
            if not path.is_file() or path.name == "bijux-atlas-scripts":
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            if "DEPRECATED:" not in text:
                continue
            rel = path.relative_to(ROOT).as_posix()
            shim_paths.append(rel)
            if rel not in known:
                errors.append(f"shim missing expiry metadata: {rel}")
    if len(shim_paths) > max_active:
        errors.append(f"shim budget exceeded: active={len(shim_paths)} max_active_shims={max_active}")

    today = date.today()
    for row in shims:
        rel = row.get("path", "")
        if not rel:
            errors.append("shim metadata missing path")
            continue
        if not str(row.get("replacement", "")).strip():
            errors.append(f"shim metadata missing replacement command: {rel}")
        if not str(row.get("migration_doc", "")).strip():
            errors.append(f"shim metadata missing migration_doc: {rel}")
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
