# bijux-atlas-cli

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

End-user Atlas CLI for dataset/catalog/ingest/query contract workflows.

## Install and Use
- Local binary: `cargo run -p bijux-atlas-cli --bin bijux-atlas -- <command>`
- Examples:
  - `bijux-atlas dataset verify --root <dir> --release 110 --species homo_sapiens --assembly GRCh38`
  - `bijux-atlas catalog validate <path>`
  - `bijux-atlas openapi generate --out configs/openapi/v1/openapi.generated.json`

## Command Surface
- Stable command map: [docs/CLI_COMMAND_LIST.md](docs/CLI_COMMAND_LIST.md)
- UX and output contract: [docs/CLI_UX_CONTRACT.md](docs/CLI_UX_CONTRACT.md)
- Exit codes: [docs/EXIT_CODES.md](docs/EXIT_CODES.md)

## Stability Guarantees
- Global flags and noun-first product commands are stable.
- `--json` emits deterministic canonical JSON for CI snapshots.
- Error payloads and exit code classes are stable contracts.

## Docs
- [docs/INDEX.md](docs/INDEX.md)
