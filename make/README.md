# Make

`make` is the minimal wrapper surface for common repository automation.

## Scope

- Public targets are the names listed in `make/root.mk:CURATED_TARGETS`.
- Each public target must be a thin wrapper over `bijux-dev-atlas` or an approved cargo-native lane in `make/cargo.mk`.
- Complex orchestration belongs in Rust, not in make recipes.

## Sources Of Truth

- Public target list: `make/root.mk`
- Target metadata and workflow usage: `configs/sources/operations/ops/make-target-registry.json`
- Generated public target artifact: `make/target-list.json`

## Documents

- Operator guide: `docs/06-development/automation-control-plane.md`
- Command reference: `docs/07-reference/automation-command-surface.md`

## Internal Support Targets

- `make-target-list` regenerates `make/target-list.json` for workflows and audits.
- Contract and lane wrappers outside `make help` are support entrypoints for CI and focused validation.
