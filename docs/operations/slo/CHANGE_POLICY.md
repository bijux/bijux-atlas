# SLO Change Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Any change to `configs/ops/slo/slo.v1.json` must include one of:

- a linked ADR in `docs/adrs/`, or
- an entry in `docs/operations/slo/CHANGELOG.md`.

SLO policy remains `v1`-compatible unless explicitly version-bumped.
