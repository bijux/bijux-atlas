# atlasctl

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
- CI/local scripts gate runs the canonical scripts suite via `make scripts-check`.

## Usage
- `make atlasctl/internal/deps/sync`
- `make scripts-run CMD="doctor --json"`
- `make scripts-check`
- `make scripts-test`
- `./bin/atlasctl inventory all --format both --out-dir docs/_generated`

### Command Surface
- `doctor`
- `check` (repository checks and policy gates)
- `gen` (report and metadata generation helpers)
- `ci` (CI-facing wrappers and validations)
- `release` (release workflow wrappers)
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

See `packages/atlasctl/PUBLIC_API.md` for current boundaries.
See `packages/atlasctl/ARCHITECTURE.md` and `docs/atlasctl/BOUNDARIES.md` for architecture SSOT and hard boundary policies.

## Control-Plane Migration Plan
- Inventory source of truth: `docs/_generated/inventory-all.json` and `docs/_generated/inventory-all.md`.
- Classification buckets: `library_helper`, `report_emitter`, `gate_runner`, `ops_orchestrator`, `docs_generator`, `config_validator`, `policy_checker`, `make_integration`.
- Porting order: `configs` commands first, then `make/layout`, then `docs/policy`, then remaining ops/public scripts.
- Migration rule: every moved command must expose deterministic output and be callable via `./bin/atlasctl <domain> ...`.
- Migration gate: direct legacy script-tree path calls from make recipes are controlled by `atlasctl check make-scripts-refs` with dated exceptions in `configs/layout/make-scripts-reference-exceptions.json`.

## Current Port Status
| Legacy script | Package command |
|---|---|
| legacy config-print shim | `./bin/atlasctl configs print` |
| legacy config-drift shim | `./bin/atlasctl configs drift` |
| legacy config-validate shim | `./bin/atlasctl configs validate` |
| legacy ops-policy-audit shim | `./bin/atlasctl ops policy-audit` |
| legacy k8s-flakes shim | `./bin/atlasctl ops k8s-flakes-check` |
| legacy k8s-test-contract shim | `./bin/atlasctl ops k8s-test-contract` |
