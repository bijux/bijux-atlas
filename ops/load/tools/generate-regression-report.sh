#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: generate-regression-report.sh <baseline.json> <candidate.json> <output.json>" >&2
  exit 1
fi

baseline="$1"
candidate="$2"
output="$3"

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
tmp_regressions="$(mktemp)"
trap 'rm -f "$tmp_regressions"' EXIT

"${repo_root}/load/tools/detect-performance-regression.sh" "$baseline" "$candidate" > "${tmp_regressions}"
classification="$("${repo_root}/load/tools/classify-regression.sh" "${tmp_regressions}")"

jq -n \
  --arg baseline_path "$baseline" \
  --arg candidate_path "$candidate" \
  --argjson classification "$classification" \
  '{
    schema_version: 1,
    kind: "performance_regression_report_v1",
    baseline_path: $baseline_path,
    candidate_path: $candidate_path,
    generated_at_utc: (now | todateiso8601),
    summary: {
      total_regressions: ($classification.total // 0),
      critical: (($classification.rows // []) | map(select(.severity=="critical")) | length),
      warning: (($classification.rows // []) | map(select(.severity=="warning")) | length),
      info: (($classification.rows // []) | map(select(.severity=="info")) | length)
    },
    regressions: ($classification.rows // [])
  }' > "${output}"
