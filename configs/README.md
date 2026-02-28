# Configurations

`configs/` is the canonical home for repository configuration inputs.

The authoritative machine registry is `configs/inventory/configs.json`. Contracts are listed in `configs/CONTRACT.md` and enforced by `bijux dev atlas contracts configs`.

Operational rules:
- Root markdown is limited to this file and `configs/CONTRACT.md`.
- `configs/docs/` is a tooling-only directory for docs linters and pinned tool inputs, not narrative documentation.
- Configuration ownership, visibility, and depth budgets are defined in the configs registry.
- Contracts output is the primary evidence. Human docs are secondary.

Common commands:
- `bijux dev atlas contracts configs --format table`
- `bijux dev atlas contracts all --format json`
- `bijux dev atlas configs inventory --format json`
