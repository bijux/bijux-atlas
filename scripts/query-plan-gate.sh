#!/usr/bin/env bash
set -euo pipefail

cargo test -p bijux-atlas-query explain_plan_snapshots_by_query_class -- --nocapture
cargo test -p bijux-atlas-query no_table_scan_assertion_for_indexed_query_plan -- --nocapture
