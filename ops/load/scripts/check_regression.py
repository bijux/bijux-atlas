#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import os
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
thresholds = json.loads((root / "configs/perf/k6-thresholds.v1.json").read_text())
baseline = json.loads((root / "artifacts/perf/baseline.json").read_text())
profile = os.environ.get("ATLAS_PERF_BASELINE_PROFILE", "").strip()
if profile:
    baseline_file = root / "ops/load/baselines" / f"{profile}.json"
    if baseline_file.exists():
        baseline = json.loads(baseline_file.read_text())
    else:
        print(f"baseline profile not found: {baseline_file}", file=sys.stderr)
        sys.exit(1)

violations = []
for row in baseline.get("rows", []):
    t = thresholds.get(row["suite"])
    if not t:
        continue
    if row["p95_ms"] > t["p95_ms"]:
        violations.append(f"{row['suite']}: p95 {row['p95_ms']:.2f} > {t['p95_ms']}")
    if row["p99_ms"] > t["p99_ms"]:
        violations.append(f"{row['suite']}: p99 {row['p99_ms']:.2f} > {t['p99_ms']}")
    if row["fail_rate"] > t["fail_rate"]:
        violations.append(f"{row['suite']}: fail_rate {row['fail_rate']:.4f} > {t['fail_rate']}")

global_t = thresholds.get("__global__", {})
cold_start = baseline.get("cold_start", {})
if cold_start:
    cold_start_ms = cold_start.get("cold_start_ms", 0)
    budget = global_t.get("cold_start_ms")
    if budget is not None and cold_start_ms > budget:
        violations.append(f"cold_start: cold_start_ms {cold_start_ms} > {budget}")

soak = baseline.get("soak_memory", {})
if soak:
    growth = soak.get("growth_bytes", 0)
    budget = global_t.get("soak_memory_growth_bytes")
    if budget is not None and growth > budget:
        violations.append(f"soak_memory: growth_bytes {growth} > {budget}")

if violations:
    out = root / "artifacts/perf/regression.txt"
    out.write_text("\n".join(violations) + "\n")
    print("performance regression detected:")
    for v in violations:
        print(f"- {v}")
    sys.exit(1)

print("performance thresholds passed")
