#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
# shellcheck source=/dev/null
. "$REPO_ROOT/ops/_lib/k8s/k8s-test-common.sh"
