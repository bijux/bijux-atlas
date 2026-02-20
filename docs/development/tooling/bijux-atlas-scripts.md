# bijux-atlas-scripts

`bijux-atlas` is the canonical Python tooling CLI surface for repository checks and report helpers.

## Module Architecture
- `core`: run context, logging, filesystem write policy, schema helpers.
- `contracts`: schema validation helpers.
- `ops`, `make`, `docs`, `configs`, `policies`: domain modules.
- `registry`: pins and registry helpers.
- `report`: report utilities and scorecard helpers.
- `layout`: repository layout and boundary checks.

## Enforcement
- The module import graph is enforced by native `atlasctl check` gates.
- CI/local scripts gate runs `atlasctl check all` in `make scripts-check`.

## Usage
- `make scripts-install`
- `make scripts-run CMD="doctor --json"`
- `make scripts-check`
- `make scripts-test`
- `./bin/bijux-atlas inventory scripts-migration --format both --out-dir docs/_generated`

### Command Surface
- `doctor`
- `configs print`
- `configs drift`
- `configs validate`
- `configs check`
- `ops lint`
- `ops check`
- `make help`
- `make list`
- `inventory all`
- `report collect`
- `compat list`
- `compat check`
- `commands --json`

See `packages/bijux-atlas-scripts/PUBLIC_API.md` for current boundaries.

## Scripts Migration Plan
- Inventory source of truth: `docs/_generated/scripts-migration.json` and `docs/_generated/scripts-migration.md`.
- Classification buckets: `library_helper`, `report_emitter`, `gate_runner`, `ops_orchestrator`, `docs_generator`, `config_validator`, `policy_checker`, `make_integration`.
- Porting order: `configs` commands first, then `make/layout`, then `docs/policy`, then remaining ops/public scripts.
- Migration rule: every moved command must expose deterministic output and be callable via `bijux-atlas <domain> ...`.
- Migration gate: direct `scripts/` calls from make recipes are controlled by `atlasctl check make-scripts-refs` with dated exceptions in `configs/layout/make-scripts-reference-exceptions.json`.

## Current Port Status
| Legacy script | Package command |
|---|---|
| `scripts/areas/public/config-print.py` | `bijux-atlas configs print` |
| `scripts/areas/public/config-drift-check.py` | `bijux-atlas configs drift` |
| `scripts/areas/public/config-validate.py` | `bijux-atlas configs validate` |
| `scripts/areas/public/ops-policy-audit.py` | `bijux-atlas ops policy-audit` |
| `scripts/areas/ops/check_k8s_flakes.py` | `bijux-atlas ops k8s-flakes-check` |
| `scripts/areas/ops/check_k8s_test_contract.py` | `bijux-atlas ops k8s-test-contract` |
