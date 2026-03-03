# Add A Check

- Owner: `team:atlas-governance`
- Type: `template`
- Audience: `contributor`
- Stability: `stable`

## Checklist

1. Add the check metadata to `configs/governance/checks.registry.json`.
2. Assign one owner, one group, one stage, one severity, and deterministic tags.
3. Add the check id to `configs/governance/suites/checks.suite.json`.
4. Add or reuse a stable report schema under `configs/contracts/reports/`.
5. Make the command resolvable from the governed command inventory.
6. Run `bijux dev atlas registry doctor --fix-suggestions`.
7. Run `bijux dev atlas suites describe --suite checks`.
8. Run the check through `bijux dev atlas check run <check_id>`.

## Definition Pattern

- Pick an eternal id such as `CHECK-RUSTFMT-001`.
- Keep report file names stable and machine-readable.
- Avoid introducing more than one owner or more than one group.
