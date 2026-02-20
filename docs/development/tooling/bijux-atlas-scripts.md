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
- `./scripts/bin/bijux-atlas-scripts inventory scripts-migration --format both --out-dir docs/_generated`

See `tools/bijux-atlas-scripts/PUBLIC_API.md` for current boundaries.

## Scripts Migration Plan
- Inventory source of truth: `docs/_generated/scripts-migration.json` and `docs/_generated/scripts-migration.md`.
- Classification buckets: `library_helper`, `report_emitter`, `gate_runner`, `ops_orchestrator`, `docs_generator`, `config_validator`, `policy_checker`, `make_integration`.
- Porting order: `configs` commands first, then `make/layout`, then `docs/policy`, then remaining ops/public scripts.
- Migration rule: every moved command must expose deterministic output and be callable via `bijux-atlas-scripts <domain> ...`.

## Current Port Status
| Legacy script | Package command |
|---|---|
| `scripts/areas/public/config-print.py` | `bijux-atlas-scripts configs print` |
| `scripts/areas/public/config-drift-check.py` | `bijux-atlas-scripts configs drift` |
| `scripts/areas/public/config-validate.py` | `bijux-atlas-scripts configs validate` |
