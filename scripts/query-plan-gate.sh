#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -euo pipefail

cargo test -p bijux-atlas-query explain_plan_snapshots_by_query_class -- --nocapture
cargo test -p bijux-atlas-query no_table_scan_assertion_for_indexed_query_plan -- --nocapture