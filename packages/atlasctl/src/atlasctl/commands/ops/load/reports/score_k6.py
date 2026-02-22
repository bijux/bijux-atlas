#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import json
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
SLO = json.loads((ROOT / "configs/slo/slo.json").read_text())
SUITE_MANIFEST = json.loads((ROOT / "ops/load/suites/suites.json").read_text())
ART = ROOT / "artifacts/ops/e2e/k6"
ART.mkdir(parents=True, exist_ok=True)

violations: list[str] = []
rows: list[dict[str, float | str]] = []
manifest_by_scenario = {
    Path(s.get("scenario", "")).stem: s
    for s in SUITE_MANIFEST.get("suites", [])
    if s.get("kind") == "k6" and s.get("scenario")
}

for summary in sorted(ART.glob("*.summary.json")):
    name = summary.name.replace(".summary.json", "")
    data = json.loads(summary.read_text())
    metrics = data.get("metrics", {})
    dur = metrics.get("http_req_duration", {}).get("values", {})
    failed = metrics.get("http_req_failed", {}).get("values", {})
    p95 = float(dur.get("p(95)", 0.0))
    p99 = float(dur.get("p(99)", 0.0))
    err = float(failed.get("rate", 0.0))
    rows.append({"scenario": name, "p95_ms": p95, "p99_ms": p99, "error_rate": err})

    target = SLO.get("scenarios", {}).get(name)
    if not target:
        suite = manifest_by_scenario.get(name)
        target = (suite or {}).get("thresholds", {})
    if not target:
        continue
    if "p95_ms_max" in target and p95 > target["p95_ms_max"]:
        violations.append(f"{name}: p95 {p95:.2f} > {target['p95_ms_max']}")
    if "p99_ms_max" in target and p99 > target["p99_ms_max"]:
        violations.append(f"{name}: p99 {p99:.2f} > {target['p99_ms_max']}")
    if "error_rate_max" in target and err > target["error_rate_max"]:
        violations.append(f"{name}: error_rate {err:.4f} > {target['error_rate_max']}")

cold = ART / "cold_start.result.json"
if cold.exists():
    c = json.loads(cold.read_text())
    cold_ms = float(c.get("cold_start_ms", 0.0))
    max_cold = float(SLO.get("global", {}).get("cold_start_p99_ms_max", 0.0))
    if max_cold and cold_ms > max_cold:
        violations.append(f"cold_start: {cold_ms:.2f} > {max_cold}")

metrics_file = ART / "metrics.prom"
if metrics_file.exists():
    text = metrics_file.read_text()
    for metric in SLO.get("required_metrics", []):
        if f"{metric}" not in text:
            violations.append(f"missing metric: {metric}")
else:
    violations.append("missing metrics scrape file: artifacts/ops/e2e/k6/metrics.prom")

soak_meta = manifest_by_scenario.get("soak-30m")
soak_thresholds = (soak_meta or {}).get("thresholds", {})
if soak_thresholds and "memory_growth_bytes_max" in soak_thresholds:
    baseline_file = ROOT / "artifacts/perf/baseline.json"
    if baseline_file.exists():
        baseline = json.loads(baseline_file.read_text())
        growth = float(baseline.get("soak_memory", {}).get("growth_bytes", 0.0))
        if growth > float(soak_thresholds["memory_growth_bytes_max"]):
            violations.append(
                f"soak-30m: memory growth {growth:.0f} > {float(soak_thresholds['memory_growth_bytes_max']):.0f}"
            )

report = [
    "# E2E k6 Score Report",
    "",
    "| scenario | p95(ms) | p99(ms) | error_rate |",
    "|---|---:|---:|---:|",
]
for row in rows:
    report.append(f"| {row['scenario']} | {row['p95_ms']:.2f} | {row['p99_ms']:.2f} | {row['error_rate']:.4f} |")

impact_level = "low"
if violations:
    if len(violations) >= 5:
        impact_level = "high"
    elif len(violations) >= 2:
        impact_level = "medium"
impact = {
    "level": impact_level,
    "violation_count": len(violations),
    "estimated_error_budget_impact_percent": round(min(100.0, len(violations) * 5.0), 2),
}

report.extend(
    [
        "",
        "## SLO Impact Estimate",
        "",
        f"- impact_level: `{impact['level']}`",
        f"- violation_count: `{impact['violation_count']}`",
        f"- estimated_error_budget_impact_percent: `{impact['estimated_error_budget_impact_percent']}`",
    ]
)

(ART / "slo-impact-estimate.json").write_text(json.dumps(impact, indent=2, sort_keys=True) + "\n")

if violations:
    report.extend(["", "## Violations", ""] + [f"- {v}" for v in violations])

(ART / "score.md").write_text("\n".join(report) + "\n")
if violations:
    (ART / "violations.txt").write_text("\n".join(violations) + "\n")
    print("k6 SLO violations detected")
    for violation in violations:
        print(f"- {violation}")
    sys.exit(1)

print("k6 SLO score passed")
