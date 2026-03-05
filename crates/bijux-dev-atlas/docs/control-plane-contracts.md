# Control Plane Contracts

Canonical contract narrative for `bijux-dev-atlas`.

## Scope

`bijux-dev-atlas` is the development control-plane for repository governance, checks, docs, configs, and ops automation.

## Invocation

- Local: `cargo run -p bijux-dev-atlas -- <args>`
- Installed: `bijux dev atlas <args>`

## Runtime Invariants

- Checks are pure by default.
- Effects require explicit capability flags (`--allow-write`, `--allow-subprocess`, `--allow-network`, `--allow-git`).
- Machine-readable outputs keep stable schema versions.
- Artifact writes stay under governed artifact roots.

## Registry Invariants

- Every registered check id maps to an implementation.
- Public implemented checks must have registry entries.
- Registries and selector expansion are deterministic.
- Internal and slow checks remain explicitly classified.

## Related

- `contract.md` (compatibility alias)
- `registry-contract.md` (compatibility alias)
