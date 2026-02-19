#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

PROFILE="${PROFILE:-kind}"

make -s ops-stack-up PROFILE="$PROFILE"
make -s ops-stack-up PROFILE="$PROFILE"
make -s ops-stack-down
make -s ops-stack-down

echo "stack up/down idempotency check passed"
