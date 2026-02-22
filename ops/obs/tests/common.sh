#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
. "$REPO_ROOT/packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/assets/k8s_test_common.sh"
