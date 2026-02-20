#!/usr/bin/env bash
# DIR_BUDGET_SHIM
set -euo pipefail
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/k8s/k8s-test-report.sh"
