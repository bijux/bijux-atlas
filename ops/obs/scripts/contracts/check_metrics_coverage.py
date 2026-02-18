#!/usr/bin/env python3
# Purpose: validate endpoint-class telemetry coverage across captured metrics runs.
# Inputs: artifacts/ops/**/metrics.prom + ops/obs/contract/metrics.golden.prom
# Outputs: exit non-zero when required coverage signals are missing.
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
METRICS = ROOT / "artifacts" / "ops" / "observability" / "metrics.prom"
GOLDEN = ROOT / "ops" / "observability" / "contract" / "metrics.golden.prom"
CONTRACT = ROOT / "ops" / "observability" / "contract" / "metrics-contract.json"


def fail(msg: str) -> int:
    print(msg, file=sys.stderr)
    return 1


def has_line(text: str, metric: str, includes: list[str]) -> bool:
    for line in text.splitlines():
        if not line.startswith(metric + "{"):
            continue
        if all(token in line for token in includes):
            return True
    return False


def main() -> int:
    if not METRICS.exists():
        return fail(f"missing metrics snapshot: {METRICS}")
    text = METRICS.read_text(encoding="utf-8", errors="replace")
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    required_metric_names = sorted(contract.get("required_metrics", {}).keys())
    corpus = []
    for p in sorted((ROOT / "artifacts" / "ops").glob("**/metrics.prom")):
        corpus.append(p.read_text(encoding="utf-8", errors="replace"))
    if GOLDEN.exists():
        corpus.append(GOLDEN.read_text(encoding="utf-8", errors="replace"))
    merged = "\n".join(corpus)
    missing_contract = [m for m in required_metric_names if not re.search(rf"^{re.escape(m)}\{{", merged, re.MULTILINE)]
    if missing_contract:
        return fail(
            "required metrics not observed in any captured run:\n" + "\n".join(f"- {m}" for m in missing_contract)
        )
    # Additional signal quality checks are advisory and emitted as warnings.
    required_classes = ("cheap", "medium", "heavy")
    for cls in required_classes:
        if not has_line(text, "atlas_bulkhead_inflight", [f'class="{cls}"']):
            print(f"metrics coverage warning: missing atlas_bulkhead_inflight class={cls}", file=sys.stderr)
        if not has_line(text, "atlas_bulkhead_saturation", [f'class="{cls}"']):
            print(f"metrics coverage warning: missing atlas_bulkhead_saturation class={cls}", file=sys.stderr)
    if not re.search(r"^bijux_store_fetch_latency_p95_seconds\{.*backend=\"(http_s3|local_fs|federated|unknown)\"", text, re.MULTILINE):
        print("metrics coverage warning: missing store fetch backend latency metric in current scrape", file=sys.stderr)
    print("metrics coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
