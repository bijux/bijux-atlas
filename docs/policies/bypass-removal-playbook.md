# Bypass Removal Playbook

This playbook defines the mandatory removal flow for any entry in the policy bypass registry.

## Inputs

- Bypass inventory: `./bin/atlasctl policy bypass list --report json`
- Due removals: `./bin/atlasctl policy bypass burn --due YYYY-MM-DD --report json`

## Removal Steps

1. Confirm owner and issue are still valid.
2. Implement the underlying fix that makes the bypass unnecessary.
3. Remove the bypass entry from its source file under `configs/policy/` or approved ops metadata source.
4. Run `./bin/atlasctl suite run checks-fast --report json`.
5. Run `./bin/atlasctl policy bypass list --by-expiry --report json` and verify debt reduced.

## Evidence

- Attach the policy issue link.
- Attach check report output and the updated bypass inventory snippet.

## Ops Bypass Removal

- Anchor: `#ops-bypass-removal`
- Use this anchor in `removal_plan` fields when one playbook path is shared.

## Release Readiness Bar

The next release readiness bar is met only when all of the following are true:

- zero `.sh` checks remain under `packages/atlasctl/src/atlasctl/checks`
- zero `__pycache__/` and `*.pyc` files exist in the repository
- `ops/` and `configs/` layout contracts pass in their reshaped model
- bypass inventory trend is non-increasing and milestones are not missed
