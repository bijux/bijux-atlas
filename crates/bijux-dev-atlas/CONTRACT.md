# CONTRACT (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

## Purpose

`bijux-dev-atlas` is the repository governance control-plane behind `bijux dev atlas ...`.

This file captures stable behavior expectations for command orchestration. Internal structure and
layering rules are documented in `crates/bijux-dev-atlas/ARCHITECTURE.md`.

## Behavioral Contract

- `cli` parses and dispatches only; command execution is implemented in `commands`.
- `commands` orchestrates adapters + core and must not own core validation/business rules.
- `core` is pure and deterministic; host effects are isolated behind `ports` and `adapters`.
- Machine-readable outputs are available on command families that support `--format json`.
- Filesystem writes require explicit capability flags and are constrained to artifact roots.
- Network/subprocess/git effects are opt-in and denied by default.

## Artifact Contract

- Default artifact root is repository `artifacts/`.
- Dev-atlas writes under deterministic subtrees rooted at `artifacts/atlas-dev/...`.
- Commands must not write outside approved artifact roots.

## Related Contracts

- Architecture / layering: `crates/bijux-dev-atlas/ARCHITECTURE.md`
- Command surface: `crates/bijux-dev-atlas/COMMAND_SURFACE.md`
- Error taxonomy: `crates/bijux-dev-atlas/ERROR_TAXONOMY.md`
- Quick error map: `crates/bijux-dev-atlas/ERRORS.md`
- Usage examples: `crates/bijux-dev-atlas/EXAMPLES.md`
