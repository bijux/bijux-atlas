# bijux-dev-atlas

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Control-plane binary for repository governance under `bijux dev atlas ...`.

## Control Plane Philosophy

- No scripts as control-plane SSOT.
- Command behavior flows through crate APIs, not shell orchestration.
- Outputs are deterministic and contract-driven.
- Execution is hermetic by default: network/subprocess/write/git are opt-in.

## Stable Families

- `ops`
- `docs`
- `configs`
- `policies`
- `check`

## Common Flags

- `--json`
- `--quiet`
- `--fail-fast`
- `--repo-root`

## Contracts

- Command surface: `docs/cli-command-list.md`
- Examples and behavior: `docs/commands.md`
- Exit codes: `docs/exit-codes.md`
- Control-plane contract: `docs/contract.md`

## Crate Governance Docs

- `crates/bijux-dev-atlas/docs/architecture.md`
- `crates/bijux-dev-atlas/CONTRACT.md`
- `crates/bijux-dev-atlas/docs/errors.md`
- `crates/bijux-dev-atlas/docs/testing.md`
- `crates/bijux-dev-atlas/docs/benchmarks.md`

## Purpose
- Describe the crate responsibility and stable boundaries.

## How to use
- Read `docs/index.md` for workflows and examples.
- Use the crate through its documented public API only.

## Where docs live
- Crate docs index: `docs/index.md`
- Contract: `CONTRACT.md`
