#!/usr/bin/env python3
from __future__ import annotations

import json
import os
from datetime import datetime, timezone
from pathlib import Path
import sys


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _write_report(root: Path, target: str, marker: str, report: Path, status: str) -> None:
    payload = {
        "schema_version": 1,
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "status": status,
        "target_manifest": target,
        "quarantine_marker": marker,
    }
    schema = json.loads((root / "ops/schema/datasets/corruption-drill-report.schema.json").read_text(encoding="utf-8"))
    for key in schema.get("required", []):
        if key not in payload:
            raise SystemExit(f"corruption report missing required key: {key}")
    report.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    root = _repo_root()
    store_root = Path(os.environ.get("ATLAS_E2E_STORE_ROOT", str(root / "artifacts/e2e-store")))
    qdir = root / "artifacts/e2e-datasets/quarantine"
    qdir.mkdir(parents=True, exist_ok=True)
    target = next(iter(store_root.glob("**/manifest.json")), None)
    if target is None:
        print(f"no dataset manifest found under {store_root}", file=sys.stderr)
        return 1
    q = qdir / f"{target.parent.name}.bad"
    report = qdir / "corruption-drill-report.json"
    if q.exists():
        print(f"quarantine marker present, skipping repeated retry: {q}")
        _write_report(root, str(target), str(q), report, "skipped")
        return 0
    with target.open("a", encoding="utf-8") as fh:
        fh.write("\n#corrupted\n")
    try:
        json.loads(target.read_text(encoding="utf-8"))
        _write_report(root, str(target), str(q), report, "failed")
        print("corruption drill failed: target still valid json", file=sys.stderr)
        return 1
    except Exception:
        q.write_text(f"{datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')} {target}\n", encoding="utf-8")
        _write_report(root, str(target), str(q), report, "quarantined")
        print(f"corruption detected and quarantined: {q}")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
