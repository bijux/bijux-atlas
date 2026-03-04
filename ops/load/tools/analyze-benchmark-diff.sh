#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: analyze-benchmark-diff.sh <baseline.json> <candidate.json>" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
"${repo_root}/load/tools/compare-load-report.sh" "$1" "$2"
