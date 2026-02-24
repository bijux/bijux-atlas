# Migration Status

`bijux dev atlas` is the active development control plane for this repository.

`atlasctl` migration work remains in progress until `packages/atlasctl/` is deleted and repo-wide references are removed. Track deletion readiness through the atlasctl deletion plan and repo checks.

## Commands

- `cargo run -p bijux-dev-atlas -- check doctor --format json`
- `cargo run -p bijux-dev-atlas -- check run --suite ci_fast --format json`
- `cargo run -p bijux-dev-atlas -- ops validate --format json`
- `cargo run -p bijux-dev-atlas -- docs verify-contracts --format json`
- `cargo run -p bijux-dev-atlas -- configs validate --format json`

## Policy

- New control-plane automation must be implemented in Rust crates and exposed through `bijux dev atlas ...`.
- Makefiles and workflows are thin wrappers and must delegate to `bijux dev atlas ...` or `cargo`.
- New legacy control-plane references are forbidden.
