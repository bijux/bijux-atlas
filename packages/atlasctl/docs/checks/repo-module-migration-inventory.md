# Repo Check Migration Inventory

## Scope

Canonical check definition surfaces are:

- `packages/atlasctl/src/atlasctl/checks/domains/repo.py`
- `packages/atlasctl/src/atlasctl/checks/domains/policies.py`
- `packages/atlasctl/src/atlasctl/checks/domains/ops.py`
- `packages/atlasctl/src/atlasctl/checks/tools/`

`packages/atlasctl/src/atlasctl/checks/repo/` is in migration and should trend toward compatibility-only wrappers, then removal.

## Classification

### Rule Checks (domain-owned)

- `checks/repo/contracts/*.py`
- `checks/repo/enforcement/**/*.py`
- `checks/repo/layout/*.py`
- `checks/repo/domains/*.py`
- `checks/repo/native/modules/*.py`
- `checks/repo/native/runtime_modules/*.py`
- `checks/repo/native/workflow_contracts.py`

Target ownership:

- Repo contracts and architecture checks: `checks/domains/repo.py`
- Policy and make governance checks: `checks/domains/policies.py`
- Ops contract checks: `checks/domains/ops.py`

### Reusable Tools (shared helpers)

- `checks/tools/reachability.py` (migrated from `checks/repo/reachability.py`)
- Root shape policy data: `checks/tools/root_policy.json`

Potential additional tool extraction candidates:

- path and tree walkers in `checks/repo/enforcement/structure/`
- import graph readers in `checks/repo/enforcement/import_policy.py`
- shared file policy parsing in `checks/repo/contracts/*`

## Completed Moves

- Removed `checks/repo/native/runtime.py` side-effect loader.
- Removed `checks/repo/native_lint.py` and inlined its checks into `checks/repo/native/runtime_modules/repo_native_runtime_policies.py`.
- Removed `checks/repo/reachability.py`; canonical implementation now lives in `checks/tools/reachability.py`.

## Remaining Migration Work

- Move check registration source of truth from `checks/repo/__init__.py` into domain modules.
- Replace `checks/repo/__init__.py` with a temporary compatibility façade, then remove.
- Regenerate registry artifacts to replace stale `module = "atlasctl.checks.repo.*"` entries.
