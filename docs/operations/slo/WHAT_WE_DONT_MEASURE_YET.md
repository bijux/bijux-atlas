# What We Do Not Measure Yet

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Native p99 request-duration metric per endpoint class (`bijux_http_request_latency_p99_seconds`).
- Direct registry refresh age gauge (`bijux_registry_refresh_age_seconds`).
- Explicit cold-start-to-first-query metric for standard endpoints.
- Explicit correctness mismatch counter between `/v1/genes/count` and `/v1/genes`.
- Per-class saturation windows for queue-depth-to-error conversion.

These gaps are declared as `planned` in `configs/ops/slo/slis.v1.json`.
