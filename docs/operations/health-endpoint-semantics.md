# Health Endpoint Semantics

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- `/healthz`: process is alive.
- `/readyz`: service is ready to serve user traffic.
- `/healthz/overload`: overload gate signal for protective throttling.

Operational checks are exercised via make targets:

```bash
$ make ops-smoke
$ make ops-drill-rate-limit
$ make ops-drill-memory-growth
```

Canonical targets: `ops-smoke`, `ops-drill-rate-limit`, `ops-drill-memory-growth`.

## Safety Valve Policy

- Emergency heavy-endpoint safety valve: set `ATLAS_DISABLE_HEAVY_ENDPOINTS=1`.
- When enabled, heavy endpoints return `503` with `QueryRejectedByPolicy` while cheap endpoints stay available.
- Validate behavior via API contracts:
  - `cargo test -p bijux-atlas-server --test api-contracts safety_valve_disables_heavy_endpoints_but_keeps_cheap_endpoints_available`
