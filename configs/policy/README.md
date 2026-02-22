# Policy Configs

Policy SSOT and relaxation registry.

- Policy: policy definition in `policy.json`
- Schema: policy schema in `policy.schema.json`
- Bypass schema (generic): `bypass.schema.json`

## Bypass Sources (sorted)
- `configs/policy/budget-relaxations.json`
- `configs/policy/check-filename-allowlist.json`
- `configs/policy/dead-modules-allowlist.json`
- `configs/policy/dependency-exceptions.json`
- `configs/policy/effect-boundary-exceptions.json`
- `configs/policy/forbidden-adjectives-allowlist.txt`
- `configs/policy/layer-live-diff-allowlist.json`
- `configs/policy/layer-relaxations.json`
- `configs/policy/ops-lint-relaxations.json`
- `configs/policy/ops-smoke-budget-relaxations.json`
- `configs/policy/pin-relaxations.json`
- `configs/policy/policy-relaxations.json`
- `configs/policy/shell-network-fetch-allowlist.txt`
- `configs/policy/shell-probes-allowlist.txt`
- `configs/policy/slow-checks-ratchet.json`

## Reporting
- List inventory: `./bin/atlasctl policies bypass list --report json`
- Write report: `./bin/atlasctl policies bypass report --out artifacts/reports/atlasctl/policies-bypass-report.json`
