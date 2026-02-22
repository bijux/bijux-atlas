#!/usr/bin/env python3
from __future__ import annotations

import json
import os
from datetime import datetime, timezone
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    out = Path(os.environ.get("OPS_RUN_DIR", str(root / "artifacts/ops/manual"))) / "promotion"
    out.mkdir(parents=True, exist_ok=True)
    source = root / "artifacts/e2e-datasets/catalog.json"
    fallback = '{"datasets":[]}\n'
    for env in ("dev", "staging", "prod"):
        target = out / f"catalog.{env}.json"
        if source.exists():
            target.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
        else:
            target.write_text(fallback, encoding="utf-8")
    (out / "catalog.staging.json").write_text((out / "catalog.dev.json").read_text(encoding="utf-8"), encoding="utf-8")
    (out / "catalog.prod.json").write_text((out / "catalog.staging.json").read_text(encoding="utf-8"), encoding="utf-8")
    dev = json.loads((out / "catalog.dev.json").read_text(encoding="utf-8"))
    count = len(dev.get("datasets", [])) if isinstance(dev, dict) else 0
    report = {
        "schema_version": 1,
        "run_id": os.environ.get("RUN_ID") or os.environ.get("OPS_RUN_ID") or "manual",
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_catalog": "catalog.dev.json",
        "environments": ["dev", "staging", "prod"],
        "promoted_count": count,
    }
    schema = json.loads((root / "ops/_schemas/datasets/promotion-report.schema.json").read_text(encoding="utf-8"))
    for key in schema.get("required", []):
        if key not in report:
            raise SystemExit(f"promotion report missing required key: {key}")
    (out / "promotion-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"promotion simulation written to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
