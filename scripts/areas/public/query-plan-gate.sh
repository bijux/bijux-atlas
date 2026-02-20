#!/usr/bin/env sh
# owner: platform
# purpose: enforce query planner index usage snapshot expectations.
# stability: public
# called-by: make query-plan-gate
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -euo pipefail

cargo test -p bijux-atlas-query explain_plan_snapshots_by_query_class -- --nocapture
cargo test -p bijux-atlas-query no_table_scan_assertion_for_indexed_query_plan -- --nocapture
./bin/atlasctl run scripts/areas/public/contracts/check_sqlite_indexes_contract.py
./bin/atlasctl run scripts/areas/public/perf/run_critical_queries.py
