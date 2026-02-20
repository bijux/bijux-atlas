# bijux-atlas-scripts

`bijux-atlas-scripts` is the Python tooling surface for repository checks and report helpers.

## Module Architecture
- `core`: run context, logging, filesystem write policy, schema helpers.
- `contracts`: schema validation helpers.
- `ops`, `make`, `docs`, `configs`, `policies`: domain modules.
- `registry`: pins and registry helpers.
- `report`: report utilities and scorecard helpers.
- `layout`: repository layout and boundary checks.

## Enforcement
- The module import graph is enforced by `scripts/areas/check/check-bijux-atlas-scripts-boundaries.py`.
- CI/local scripts gate runs this boundary check in `make scripts-check`.

## Usage
- `make scripts-install`
- `make scripts-run CMD="doctor --json"`
- `make scripts-check`
- `make scripts-test`

See `tools/bijux-atlas-scripts/PUBLIC_API.md` for current boundaries.
