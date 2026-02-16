#!/usr/bin/env python3
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[2]
thresholds = json.loads((root / "configs/perf/thresholds.json").read_text())
baseline = json.loads((root / "artifacts/perf/baseline.json").read_text())

violations = []
for row in baseline.get("rows", []):
    t = thresholds.get(row["suite"])
    if not t:
        continue
    if row["p95_ms"] > t["p95_ms"]:
        violations.append(f"{row['suite']}: p95 {row['p95_ms']:.2f} > {t['p95_ms']}")
    if row["fail_rate"] > t["fail_rate"]:
        violations.append(f"{row['suite']}: fail_rate {row['fail_rate']:.4f} > {t['fail_rate']}")

if violations:
    out = root / "artifacts/perf/regression.txt"
    out.write_text("\n".join(violations) + "\n")
    print("performance regression detected:")
    for v in violations:
        print(f"- {v}")
    sys.exit(1)

print("performance thresholds passed")
