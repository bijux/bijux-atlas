#!/usr/bin/env python3
# owner: platform
# purpose: validate spike-proof k6 suite counters and overload recovery semantics.
# stability: public
# called-by: make ops-load-spike-proof
# Purpose: enforce spike-proof expectations from summary metrics and post-spike overload recovery status.
# Inputs: k6 summary path, base URL, and optional wait timeout/cadence arguments.
# Outputs: exit 0 on pass; exit 1 with deterministic assertion failures.

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

from ...core.network import http_get


def read_counter(summary: dict, metric: str) -> float:
    values = summary.get("metrics", {}).get(metric, {}).get("values", {})
    return float(values.get("count", 0.0))


def fetch_overload_status(base_url: str) -> tuple[int, bool]:
    status, body = http_get(f"{base_url.rstrip('/')}/healthz/overload", timeout_seconds=5)
    overloaded = '"overloaded":true' in body.replace(" ", "")
    return status, overloaded


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--summary", default="artifacts/perf/results/spike-overload-proof.summary.json")
    parser.add_argument("--base-url", default="http://127.0.0.1:18080")
    parser.add_argument("--wait-seconds", type=int, default=45)
    parser.add_argument("--poll-seconds", type=int, default=3)
    args = parser.parse_args()

    summary_path = Path(args.summary)
    if not summary_path.exists():
        print(f"missing summary: {summary_path}", file=sys.stderr)
        return 1

    summary = json.loads(summary_path.read_text())
    required_positive = [
        "cheap_survival_ok_total",
        "heavy_shed_observed_total",
        "overload_active_observed_total",
        "queue_depth_metric_observed_total",
        "queue_depth_positive_observed_total",
    ]
    errors: list[str] = []
    for metric in required_positive:
        if read_counter(summary, metric) <= 0:
            errors.append(f"expected {metric} count > 0")
    if read_counter(summary, "rss_cap_exceeded_total") != 0:
        errors.append("rss cap exceeded during spike")

    deadline = time.time() + max(1, args.wait_seconds)
    recovered = False
    last_status = -1
    while time.time() < deadline:
        try:
            last_status, overloaded = fetch_overload_status(args.base_url)
            if last_status == 200 and not overloaded:
                recovered = True
                break
        except Exception:
            pass
        time.sleep(max(1, args.poll_seconds))

    if not recovered:
        errors.append(
            f"overload endpoint did not auto-clear to healthy state within {args.wait_seconds}s (last_status={last_status})"
        )

    if errors:
        print("spike-proof assertions failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("spike-proof assertions passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
