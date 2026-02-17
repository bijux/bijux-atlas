#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"
make ops-up
make ops-reset
make ops-publish-medium
make ops-deploy
make ops-k8s-tests
make ops-drill-pod-churn
make ops-report
