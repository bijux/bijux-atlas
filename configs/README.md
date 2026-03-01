# Configurations

`configs/` is the canonical home for repository configuration inputs. The authoritative machine registries are `configs/inventory/configs.json` and `configs/configs.contracts.json`.
Ownership and consumer mapping SSOT files:
- `configs/OWNERS.json`
- `configs/CONSUMERS.json`

Common commands:
- `bijux dev atlas contracts configs --format table`
- `bijux dev atlas configs list --format json`

Example config files are allowed only under `configs/examples/`.

Narrative docs belong under `docs/`, not under `configs/`.
