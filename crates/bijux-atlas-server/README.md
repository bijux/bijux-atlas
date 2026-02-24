# bijux-atlas-server

Runtime HTTP service for `bijux-atlas`.

## Local Run
- `cargo run -p bijux-atlas-server --bin atlas-server`
- Health: `GET /healthz`
- Readiness: `GET /readyz`

## Config
- Environment schema contract: `configs/contracts/env.schema.json`
- Startup config validation is fail-fast at process boot.
- Operational guide: [docs/OPERATIONS_RUNBOOK.md](docs/OPERATIONS_RUNBOOK.md)

## Stability
- Stable HTTP/OpenAPI surface is `v1`.
- Response error envelope and request-id propagation are contract behavior.

## Docs
- [docs/INDEX.md](docs/INDEX.md)
