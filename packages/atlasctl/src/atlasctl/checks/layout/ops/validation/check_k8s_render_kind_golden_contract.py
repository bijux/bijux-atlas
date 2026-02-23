#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]
GOLDEN = ROOT / "ops/k8s/tests/goldens/render-kind.summary.json"


def main() -> int:
    if not GOLDEN.exists():
        print("missing k8s render golden: ops/k8s/tests/goldens/render-kind.summary.json", file=sys.stderr)
        return 1
    payload = json.loads(GOLDEN.read_text(encoding="utf-8"))
    errs: list[str] = []
    if payload.get("kind") != "ops-k8s-render-summary":
        errs.append("render-kind.summary.json must be an ops-k8s-render-summary payload")
    if "run_id" not in payload:
        errs.append("render-kind.summary.json must contain run_id for explicit golden provenance")
    if "render_hash" not in payload:
        errs.append("render-kind.summary.json must contain render_hash")
    if "generated_at" in payload or "timestamp" in payload:
        errs.append("render-kind.summary.json must not contain nondeterministic timestamp fields")
    if not isinstance(payload.get("test_count"), int):
        errs.append("render-kind.summary.json must contain integer test_count")
    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("k8s render-kind summary golden contract valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

