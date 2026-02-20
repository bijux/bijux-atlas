# PUBLIC API

`bijux-atlas-scripts` exposes a CLI surface and internal module boundaries.

## Stable CLI Commands
- `bijux-atlas-scripts doctor`
- `bijux-atlas-scripts surface`
- `bijux-atlas-scripts validate-output`
- Domain commands: `ops`, `docs`, `configs`, `policies`, `make`, `inventory`, `contracts`, `registry`, `layout`, `report`

## Internal Modules
- `bijux_atlas_scripts.core`: run context, structured logging, filesystem evidence policy, schema helpers.
- `bijux_atlas_scripts.contracts`: contract validation helpers and schema-facing utilities.
- `bijux_atlas_scripts.ops`: ops domain orchestration entrypoint helpers.
- `bijux_atlas_scripts.make`: make target and surface helpers.
- `bijux_atlas_scripts.docs`: docs scanning and documentation contract helpers.
- `bijux_atlas_scripts.configs`: config inventory and drift checks.
- `bijux_atlas_scripts.policies`: policy enforcement and relaxation checks.
- `bijux_atlas_scripts.registry`: pins and registry policy helpers.
- `bijux_atlas_scripts.report`: unified report and scorecard helpers.
- `bijux_atlas_scripts.layout`: layout/boundary checks and structure policy helpers.

## Boundary Contract
- Cross-module imports are restricted and enforced by `scripts/areas/check/check-bijux-atlas-scripts-boundaries.py`.
- Shared code belongs in `core` or `contracts`.
- Domain modules must not depend on each other unless declared in the boundary allowlist.
