#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
. "$REPO_ROOT/ops/_lib/k8s-test-common.sh"
