# bijux-atlas-scripts

Internal Python package for script orchestration and stable script APIs.

## CLI

- `bijux-atlas-scripts run <script-path> [args...]`
- `bijux-atlas-scripts validate-output --schema <schema.json> --file <output.json>`
- `bijux-atlas-scripts surface --json`
- `bijux-atlas-scripts doctor --json`

Global context flags:
- `--run-id`
- `--evidence-root`
- `--profile`
- `--no-network`

## Modules

- `bijux_atlas_scripts.ops`
- `bijux_atlas_scripts.docs`
- `bijux_atlas_scripts.configs`
- `bijux_atlas_scripts.policies`
- `bijux_atlas_scripts.make`
- `bijux_atlas_scripts.reporting`
