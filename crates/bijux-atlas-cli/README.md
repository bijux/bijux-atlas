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
- Stable command map: [docs/cli-command-list.md](docs/cli-command-list.md)
- UX and output contract: [docs/cli-ux-contract.md](docs/cli-ux-contract.md)
- Exit codes: [docs/exit-codes.md](docs/exit-codes.md)

## Stability Guarantees
- Global flags and noun-first product commands are stable.
- `--json` emits deterministic canonical JSON for CI snapshots.
- Error payloads and exit code classes are stable contracts.

## Docs
- [docs/index.md](docs/index.md)

## Purpose
- Describe the crate responsibility and stable boundaries.

## How to use
- Read `docs/index.md` for workflows and examples.
- Use the crate through its documented public API only.

## Where docs live
- Crate docs index: `docs/index.md`
- Contract: `CONTRACT.md`
