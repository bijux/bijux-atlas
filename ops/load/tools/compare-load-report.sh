#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: compare-load-report.sh <baseline.json> <candidate.json>" >&2
  exit 1
fi

baseline="$1"
candidate="$2"

jq -n --slurpfile b "$baseline" --slurpfile c "$candidate" '
{
  schema_version: 1,
  kind: "load_report_comparison_v1",
  baseline_profile: ($b[0].profile // "unknown"),
  candidate_profile: ($c[0].profile // "unknown"),
  suites: (
    ($b[0].suites | keys_unsorted) as $keys
    | $keys
    | map({
        suite: .,
        baseline: $b[0].suites[.],
        candidate: $c[0].suites[.],
        delta_p95_ms: (($c[0].suites[.].p95_ms // 0) - ($b[0].suites[.].p95_ms // 0)),
        delta_p99_ms: (($c[0].suites[.].p99_ms // 0) - ($b[0].suites[.].p99_ms // 0)),
        delta_error_rate: (($c[0].suites[.].error_rate // 0) - ($b[0].suites[.].error_rate // 0))
      })
  )
}
'
