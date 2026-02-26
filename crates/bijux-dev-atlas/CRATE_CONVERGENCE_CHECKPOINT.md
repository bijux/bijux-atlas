<!-- SPDX-License-Identifier: Apache-2.0 -->

# Dev Atlas Crate Convergence Checkpoint

- Owner: bijux-dev-atlas maintainers
- Scope: `crates/bijux-dev-atlas` control-plane convergence
- Intent: track the in-progress merge from `bijux-dev-atlas-*` crates into one crate without schema/report drift

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

## Remaining Before Workspace Member Removal

- Replace production imports of:
  - model data contracts are also available in `crates/bijux-dev-atlas/src/model/`
  - policy schema and validation code are also available in `crates/bijux-dev-atlas/src/policies/`
  - core engine and checks are also available in `crates/bijux-dev-atlas/src/core/`
  - adapter implementations are also available in `crates/bijux-dev-atlas/src/adapters/`
  with internal module paths
- Extract `src/core/ports.rs` into top-level `src/ports/*`
- Rewire `core` to depend on `crate::ports`
- Rewire command execution paths to use internal `core` + `adapters`
- Consolidate remaining core-related tests/goldens under `crates/bijux-dev-atlas/tests/`
- Remove all references to old crate names in docs/configs/ops/CI
- Remove old crates from workspace members and workspace dependencies

## Removal Gate (Batch 7 / Batch 8)

Do not remove workspace members for `bijux-dev-atlas-core`, `-model`, `-adapters`, or `-policies` until:

1. `cargo metadata` no longer shows `bijux-dev-atlas` depending on those crates
2. `cargo test -p bijux-dev-atlas` passes using internal modules
3. `cargo bench -p bijux-dev-atlas --no-run` passes for migrated benches
4. grep for old crate names is clean in code and CI paths
