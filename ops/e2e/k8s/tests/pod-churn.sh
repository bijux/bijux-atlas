#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)"
exec "$ROOT/bin/atlasctl" run ./packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/rollout/pod_churn.py "$@"
