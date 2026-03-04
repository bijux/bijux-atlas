#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: classify-regression.sh <regressions.json>" >&2
  exit 1
fi

regressions_file="$1"

jq '
  {
    schema_version: 1,
    kind: "performance_regression_classification_v1",
    total: length,
    rows: map(
      . + {
        severity: (
          if .delta_error_rate > 0.02 or .p95_ratio > 0.2 then "critical"
          elif .delta_error_rate > 0.01 or .p95_ratio > 0.1 then "warning"
          else "info"
          end
        )
      }
    )
  }
' "${regressions_file}"
