#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"

"$ROOT/e2e/realdata/run_single_release.sh"
"$ROOT/e2e/realdata/run_two_release_diff.sh"
"$ROOT/e2e/realdata/schema_evolution.sh"
"$ROOT/e2e/realdata/upgrade_drill.sh"
"$ROOT/e2e/realdata/rollback_drill.sh"

echo "realdata suite passed"
