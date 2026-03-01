# bijux-atlas-core

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Deterministic domain primitives and boundary contracts shared across bijux-atlas crates.

## Stability

This crate defines low-level contracts. Public API changes require deliberate versioning and contract test updates.

## Public entrypoints

- `canonical` module and hashing helpers (`sha256`, `sha256_hex`, `Hash256`)
- canonical errors (`Error`, `Result<T>`, machine errors, exit codes)
- invariant identifiers (`DatasetId`, `ShardId`, `RunId`)
- effect boundary traits (`FsPort`, `ClockPort`, `NetPort`, `ProcessPort`)

## Do Not

- add runtime effects to pure domain logic
- leak raw `String` identifiers where a newtype exists
- introduce alternate top-level error types
- expand public API without updating `docs/public-api.md` and contract tests

## Architecture

See `docs/architecture.md`.

## Purpose
- Describe the crate responsibility and stable boundaries.

## How to use
- Read `docs/index.md` for workflows and examples.
- Use the crate through its documented public API only.

## Where docs live
- Crate docs index: `docs/index.md`
- Contract: `CONTRACT.md`
