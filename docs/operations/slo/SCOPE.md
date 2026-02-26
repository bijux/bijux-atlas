# SLO Scope

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## In scope

- API availability.
- Request success rate.
- Latency per endpoint class.
- Overload behavior (`cheap` class survival under shedding).
- Dataset freshness and registry refresh success.

## Endpoint classes (SSOT)

- `cheap`: `health`, `version`, `metrics`.
- `standard`: `genes list`, `genes by id`.
- `heavy`: `diff`, `region`, `sequence`.
