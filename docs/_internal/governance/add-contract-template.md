# Add A Contract

- Owner: `team:atlas-governance`
- Type: `template`
- Audience: `contributor`
- Stability: `stable`

## Checklist

1. Add the contract metadata to `configs/governance/contracts.registry.json`.
2. Assign one owner, one contract group, and one runner surface.
3. Add the contract id to `configs/governance/suites/contracts.suite.json`.
4. Declare stable report artifacts.
5. Make the runner resolvable from the governed command inventory.
6. Run `bijux dev atlas registry doctor --fix-suggestions`.
7. Run `bijux dev atlas contract run <contract_id>`.
8. Run `bijux dev atlas suites run --suite contracts --group <group>`.

## Definition Pattern

- Prefer domain-owned ids such as `OPS-K8S-001`.
- Keep one source of truth for the runner.
- Keep report paths durable across releases.
