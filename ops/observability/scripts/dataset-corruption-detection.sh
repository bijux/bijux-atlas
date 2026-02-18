#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: validate corrupted dataset is detected and quarantined by cache manager checks.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
cargo test -p bijux-atlas-server cache_manager_tests::chaos_mode_random_byte_corruption_never_serves_results -- --exact
echo "dataset corruption detection drill passed"
