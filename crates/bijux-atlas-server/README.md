# bijux-atlas-server

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Runtime HTTP service for `bijux-atlas`.

## Local Run
- `cargo run -p bijux-atlas-server --bin atlas-server`
- Health: `GET /healthz`
- Readiness: `GET /readyz`

## Config
- Environment schema contract: `configs/contracts/env.schema.json`
- Startup config validation is fail-fast at process boot.
- Operational guide: [docs/operations-runbook.md](docs/operations-runbook.md)

## Stability
- Stable HTTP/OpenAPI surface is `v1`.
- Response error envelope and request-id propagation are contract behavior.

## Docs
- [docs/index.md](docs/index.md)
