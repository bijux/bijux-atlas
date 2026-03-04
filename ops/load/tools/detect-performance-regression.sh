#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: detect-performance-regression.sh <baseline.json> <candidate.json>" >&2
  exit 1
fi

baseline="$1"
candidate="$2"

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
diff_json="$("${repo_root}/load/tools/analyze-benchmark-diff.sh" "$baseline" "$candidate")"

latency_ratio_max="$(jq -r '.thresholds.latency_regression_ratio_max' "${repo_root}/../configs/perf/regression-policy.json")"
error_abs_max="$(jq -r '.thresholds.error_rate_regression_abs_max' "${repo_root}/../configs/perf/regression-policy.json")"

regressions="$(echo "${diff_json}" | jq --argjson latency_ratio_max "${latency_ratio_max}" --argjson error_abs_max "${error_abs_max}" '
  [
    .suites[]
    | . as $row
    | ($row.baseline.p95_ms // 0) as $baseline_p95
    | ($row.delta_p95_ms // 0) as $delta_p95
    | ($row.delta_error_rate // 0) as $delta_error
    | {
        suite: $row.suite,
        p95_ratio: (if $baseline_p95 > 0 then ($delta_p95 / $baseline_p95) else 0 end),
        delta_error_rate: $delta_error
      }
    | select(.p95_ratio > $latency_ratio_max or .delta_error_rate > $error_abs_max)
  ]')"

echo "${regressions}"
