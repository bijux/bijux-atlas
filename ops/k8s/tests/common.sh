#!/usr/bin/env bash
set -euo pipefail
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)/ops/e2e/k8s/tests/common.sh"
