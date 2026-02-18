#!/usr/bin/env python3
# Purpose: validate endpoint-class telemetry coverage from captured metrics snapshot.
# Inputs: artifacts/ops/observability/metrics.prom
# Outputs: exit non-zero when required coverage signals are missing.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
METRICS = ROOT / "artifacts" / "ops" / "observability" / "metrics.prom"


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
    required_classes = ("cheap", "medium", "heavy")
    for cls in required_classes:
        if not has_line(text, "atlas_bulkhead_inflight", [f'class="{cls}"']):
            return fail(f"missing atlas_bulkhead_inflight class={cls}")
        if not has_line(text, "atlas_bulkhead_saturation", [f'class="{cls}"']):
            return fail(f"missing atlas_bulkhead_saturation class={cls}")
    required_shed_reasons = (
        "queue_depth_exceeded",
        "class_permit_saturated",
        "bulkhead_shed_noncheap",
    )
    for reason in required_shed_reasons:
        if not has_line(text, "atlas_shed_total", [f'reason="{reason}"']):
            return fail(f"missing atlas_shed_total reason={reason}")
    if not has_line(text, "bijux_http_request_size_p95_bytes", ['route="/v1/genes"']):
        return fail("missing request-size p95 coverage for /v1/genes")
    if not has_line(text, "bijux_http_response_size_p95_bytes", ['route="/v1/genes"']):
        return fail("missing response-size p95 coverage for /v1/genes")
    if not re.search(r"^bijux_store_fetch_latency_p95_seconds\{.*backend=\"(http_s3|local_fs|federated|unknown)\"", text, re.MULTILINE):
        return fail("missing store fetch backend latency coverage metric")
    print("metrics coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
