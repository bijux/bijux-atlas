# Ops System Gates

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Gate definitions are canonical in:
- `ops/inventory/gates.json`
- `ops/inventory/registry.toml`

Execution entrypoints:
- `bijux dev atlas check registry doctor --format json`
- `bijux dev atlas ops validate --profile kind --format json`
