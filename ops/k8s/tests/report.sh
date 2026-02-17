#!/usr/bin/env bash
set -euo pipefail
"$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)/ops/_lib/k8s-test-report.sh" "$@"
