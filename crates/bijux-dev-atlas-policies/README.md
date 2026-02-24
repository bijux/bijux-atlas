# bijux-dev-atlas-policies

Governance policy crate for `bijux-dev-atlas`.

## Scope
- Repository governance policies only (repo shape, ops SSOT paths, check routing constraints).
- Runtime query/ingest/API budgets are out of scope and remain in `bijux-atlas-policies`.

## Allowed dependencies
- `serde`
- `serde_json`

## Forbidden dependencies
- Async/web runtime dependencies: `tokio`, `axum`, `hyper`.
- Runtime atlas crates: `bijux-atlas-ingest`, `bijux-atlas-store`, `bijux-atlas-query`, `bijux-atlas-server`, `bijux-atlas-api`.
