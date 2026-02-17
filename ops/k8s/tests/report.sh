#!/usr/bin/env bash
set -euo pipefail
"$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)/ops/e2e/k8s/tests/report.sh" "$@"
