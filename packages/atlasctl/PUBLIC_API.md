# PUBLIC API

`bijux-atlas` exposes a CLI surface and internal module boundaries.

## Stable CLI Commands
- `bijux-atlas doctor`
- `bijux-atlas surface`
- `bijux-atlas validate-output`
- Domain commands: `ops`, `docs`, `configs`, `policies`, `make`, `inventory`, `contracts`, `registry`, `layout`, `report`

## Internal Modules
- `atlasctl.core`: run context, structured logging, filesystem evidence policy, schema helpers.
- `atlasctl.contracts`: contract validation helpers and schema-facing utilities.
- `atlasctl.ops`: ops domain orchestration entrypoint helpers.
- `atlasctl.make`: make target and surface helpers.
- `atlasctl.docs`: docs scanning and documentation contract helpers.
- `atlasctl.configs`: config inventory and drift checks.
- `atlasctl.policies`: policy enforcement and relaxation checks.
- `atlasctl.registry`: pins and registry policy helpers.
- `atlasctl.report`: unified report and scorecard helpers.
- `atlasctl.layout`: layout/boundary checks and structure policy helpers.

## Boundary Contract
- Cross-module imports are restricted and enforced by `scripts/areas/check/check-bijux-atlas-boundaries.py`.
- Shared code belongs in `core` or `contracts`.
- Domain modules must not depend on each other unless declared in the boundary allowlist.
