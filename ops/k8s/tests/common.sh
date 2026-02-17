#!/usr/bin/env bash
set -euo pipefail
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)/ops/_lib/k8s-test-common.sh"
