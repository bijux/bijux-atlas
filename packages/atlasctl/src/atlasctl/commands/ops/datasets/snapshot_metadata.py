#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else "missing"


def main() -> int:
    root = _repo_root()
    out = Path(os.environ.get("OPS_RUN_DIR", str(root / "artifacts/ops/manual"))) / "datasets"
    out.mkdir(parents=True, exist_ok=True)
    meta = {
        "manifest_lock_sha256": _sha(root / "ops/datasets/manifest.lock"),
        "catalog_sha256": _sha(root / "artifacts/e2e-datasets/catalog.json"),
        "qc_report_sha256": _sha(root / "artifacts/e2e-datasets/qc_report.json"),
    }
    target = out / "metadata.snapshot.json"
    target.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
