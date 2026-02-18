#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)"
exec "$ROOT/ops/k8s/tests/report.sh" "$@"
