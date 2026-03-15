# Fixture Contract

- Area: `ops/datasets/fixtures`
- schema_version: `1`
- contract_version: `1.0.0`
- Owner: `bijux-atlas-operations`
- Parent contract: `ops/datasets/CONTRACT.md`
- Purpose: `fixture asset layout and governance for committed dataset samples`

## Invariants
- Fixture roots are versioned under `<fixture-name>/v<integer>/`.
- Downloadable binary payloads live only under `assets/` and use `.tar.gz`.
- Every committed fixture version has a `manifest.lock`.
- Generated inventories and drift reports describe this tree; they do not replace it.

## Enforcement
- `checks_ops_fixture_governance`
- `checks_ops_domain_contract_structure`
