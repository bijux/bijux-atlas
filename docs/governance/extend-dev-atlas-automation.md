# Extend Dev Atlas Automation

## Required workflow

1. Add a CLI surface in `crates/bijux-dev-atlas/src/cli/`.
2. Implement execution logic in runtime entry/command handlers.
3. Add parser and behavior tests under `crates/bijux-dev-atlas/tests/`.
4. Add governance docs and usage examples.
5. Expose through Make wrappers only as a single-line delegation.

## Prohibited workflow

1. Do not add root scripts.
2. Do not add `control-plane/` or `automation/`.
3. Do not add workflow shell pipelines that bypass `bijux-dev-atlas`.

## Acceptance checks

1. `bijux-dev-atlas checks automation-boundaries`
2. `bijux-dev-atlas contract automation-boundaries`
