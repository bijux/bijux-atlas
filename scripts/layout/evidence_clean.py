#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RETENTION = ROOT / "configs" / "ops" / "artifact-retention.json"
EVIDENCE_ROOT = ROOT / "artifacts" / "evidence"


def load_policy() -> tuple[int, int]:
    if not RETENTION.exists():
        return 14, 10
    payload = json.loads(RETENTION.read_text(encoding="utf-8"))
    return int(payload.get("evidence_retention_days", 14)), int(payload.get("evidence_keep_last_per_area", 10))


def main() -> int:
    days, keep_last = load_policy()
    cutoff = datetime.now(timezone.utc) - timedelta(days=days)
    if not EVIDENCE_ROOT.exists():
        print("no artifacts/evidence directory")
        return 0
    removed = 0
    for area_dir in sorted(EVIDENCE_ROOT.iterdir()):
        if not area_dir.is_dir():
            continue
        run_dirs = sorted((p for p in area_dir.iterdir() if p.is_dir()), key=lambda p: p.stat().st_mtime, reverse=True)
        if not run_dirs:
            continue
        for idx, path in enumerate(run_dirs):
            too_old = datetime.fromtimestamp(path.stat().st_mtime, tz=timezone.utc) < cutoff
            if idx < keep_last and not too_old:
                continue
            if idx < keep_last and too_old:
                continue
            if not too_old:
                continue
            shutil.rmtree(path, ignore_errors=True)
            removed += 1
            print(f"removed {path.relative_to(ROOT)}")
    print(
        "evidence cleanup complete "
        f"(removed={removed}, retention_days={days}, keep_last_per_area={keep_last})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
