# Scripting Architecture Contract (Historical Transition Note)

- Owner: `bijux-atlas-platform`
- Stability: `stable`

## What

Defines historical scripting constraints and the current Rust control-plane replacement contract.

## Current Contract (Locked)

1. Repository automation entrypoints are Rust-native and routed through `bijux dev atlas ...`.
2. Runtime product CLI commands are routed through `bijux atlas ...`.
3. `bijux dev atlas` is retired and scheduled for removal; see `docs/development/tooling/bijux dev atlas-deletion-plan.md`.
4. No direct `python` or `bash` path invocations are allowed in make/workflow/docs control-plane surfaces.
5. Runtime evidence is non-committed and must write under ignored artifact roots.
6. Control-plane artifacts use `artifacts/<run-kind>/<run-id>/...` (current dev-atlas layout: `artifacts/atlas-dev/<domain>/<run-id>/...`).
7. Deterministic generated outputs can be committed only when timestamp-free.
8. Runtime logs, lane reports, and run evidence must never be committed.

## Legacy Taxonomy (Historical)

Stable command families:
- `doctor`
- `report`
- `check`
- `gen`
- `ops`
- `docs`
- `ci`
- `release`

Legacy implementations mapped these through `bijux dev atlas` namespaces. Active control-plane ownership is now `bijux dev atlas` command groups.

## Internal command policy

- Internal commands may exist for maintainers but must be excluded from default user documentation.
- Internal commands must still honor run context, schema contracts, and output policies.

## Toolchain Contract (Historical Legacy Note)

- Legacy bijux dev atlas Python toolchain contracts are being removed with `crates/bijux-dev-atlas`.
- Active dev governance control-plane commands are Rust-native (`bijux-dev-atlas`).
- Python tooling documents are removed from the active docs surface.

## How to verify

```bash
make gates
make check
make docs
```

Expected output: policy and command-surface checks pass without direct retired script invocations.
