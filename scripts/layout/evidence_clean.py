#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RETENTION = ROOT / "configs" / "ops" / "artifact-retention.json"
EVIDENCE_MAKE = ROOT / "ops" / "_evidence" / "make"


def load_days() -> int:
    if not RETENTION.exists():
        return 14
    payload = json.loads(RETENTION.read_text(encoding="utf-8"))
    return int(payload.get("evidence_retention_days", 14))


def main() -> int:
    days = load_days()
    cutoff = datetime.now(timezone.utc) - timedelta(days=days)
    if not EVIDENCE_MAKE.exists():
        print("no ops/_evidence/make directory")
        return 0
    removed = 0
    for path in sorted(EVIDENCE_MAKE.iterdir()):
        if not path.is_dir():
            continue
        if path.name == "root-local":
            continue
        if datetime.fromtimestamp(path.stat().st_mtime, tz=timezone.utc) < cutoff:
            shutil.rmtree(path, ignore_errors=True)
            removed += 1
            print(f"removed {path.relative_to(ROOT)}")
    print(f"evidence cleanup complete (removed={removed}, retention_days={days})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
