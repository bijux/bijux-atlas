# Profile Under Load (Rust Async)

## Goal
Identify CPU and async contention hot spots while replaying reproducible k6 traffic.

## Steps
1. Start atlas server with realistic dataset cache and enable tracing.
2. Run load: `ATLAS_BASE=http://127.0.0.1:3000 k6 run load/k6/atlas_phase11.js`.
3. Capture flamegraph:
   - macOS: `sudo samply record -- cargo run -p bijux-atlas-server --bin atlas-server`
   - Linux: `perf record -F 99 -g -- <atlas-server-cmd>` then `perf script | inferno-flamegraph > flame.svg`
4. Correlate with metrics:
   - `atlas_http_request_latency_p95_seconds`
   - `atlas_sqlite_query_latency_p95_seconds`
   - `atlas_store_download_p95_seconds`
5. Validate regressions by rerunning identical load script and comparing p95 and error rate.

## Rules
- Keep dataset and query mix unchanged between profile runs.
- Record tool versions and git commit in profile notes.
