<!-- SPDX-License-Identifier: Apache-2.0 -->

# Dev Atlas Crate Convergence Checkpoint

- Owner: bijux-dev-atlas maintainers
- Scope: `crates/bijux-dev-atlas` control-plane convergence
- Intent: track convergence into one crate without schema/report drift

## Target

- Final crate: `crates/bijux-dev-atlas`
- Final binary: `bijux dev atlas ...`
- Internal module boundaries:
  - `model`
  - `policies`
  - `core`
  - `ports`
  - `adapters`
  - `cli`
  - `commands`

## Completed Internal Imports

- `model` source imported into `src/model/`
- `policies` source imported into `src/policies/`
- `core` source imported into `src/core/`
- `adapters` source imported into `src/adapters/`

## Completed Validation Coverage Migration

- `report_codec` benchmark moved to `crates/bijux-dev-atlas/benches/report_codec.rs`
- `policy_eval` benchmark moved to `crates/bijux-dev-atlas/benches/policy_eval.rs`
- `core_engine` benchmark moved to `crates/bijux-dev-atlas/benches/core_engine.rs`
- `file_walk` benchmark moved to `crates/bijux-dev-atlas/benches/file_walk.rs`
- model and policy tests/goldens copied into `crates/bijux-dev-atlas/tests/`
- boundary checks added in `tests/boundaries.rs`

## Completed Layout Convergence

- dispatch implementation lives at `src/commands/dispatch.rs`
- command handlers moved under `src/commands/` and loaded by `main.rs` via `#[path = ...]`

## Convergence Notes

- Internal module imports are wired through the unified crate.
- Ports live in `src/ports/` with core compatibility re-exports.
- Bench and test targets run from `crates/bijux-dev-atlas`.
- Remaining work focuses on repository-wide cleanup, boundary enforcement, and verification.
