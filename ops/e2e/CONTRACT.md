# Contract

- Area: `ops/e2e`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `behavioral`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/e2e/suites/suites.json` | Authored E2E suite catalog |
| `ops/e2e/scenarios/scenarios.json` | Authored scenario composition map |
| `ops/e2e/expectations/expectations.json` | Authored expectations |
| `ops/e2e/reproducibility-policy.json` | Authored reproducibility policy |
| `ops/e2e/fixtures/allowlist.json` | Authored fixture allowlist |
| `ops/e2e/taxonomy.json` | Authored scenario taxonomy |
| `ops/e2e/generated/e2e-summary.json` | Generated run summary |
| `ops/e2e/generated/coverage-matrix.json` | Generated coverage matrix |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/e2e/suites/suites.json` | `ops/schema/e2e-suites.schema.json` |
| `ops/e2e/scenarios/scenarios.json` | `ops/schema/e2e-scenarios.schema.json` |
| `ops/e2e/expectations/expectations.json` | `ops/schema/e2e/expectations.schema.json` |
| `ops/e2e/reproducibility-policy.json` | `ops/schema/e2e/reproducibility-policy.schema.json` |
| `ops/e2e/fixtures/allowlist.json` | `ops/schema/e2e/fixture-allowlist.schema.json` |
| `ops/e2e/taxonomy.json` | `ops/schema/e2e/taxonomy.schema.json` |
| `ops/e2e/generated/e2e-summary.json` | `ops/schema/e2e/summary.schema.json` |
| `ops/e2e/generated/coverage-matrix.json` | `ops/schema/e2e/coverage-matrix.schema.json` |

## Contract Taxonomy

- Structural contract: suites, scenarios, expectations, and fixture allowlists define stable composition inputs.
- Behavioral contract: reproducibility policy and generated summary/coverage outputs define runtime verification behavior.

## Invariants

- No duplicate authored truth is allowed; scenario composition is authored only in `ops/e2e/scenarios/scenarios.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- E2E is composition-only; behavior implementation does not live under `ops/e2e`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated E2E artifacts must include `generated_by` and `schema_version` metadata.
- E2E docs must be linked from `ops/e2e/INDEX.md`; orphan docs are forbidden.
- Referenced fixture assets must be versioned, allowlisted, and lock-verified.
- Suite and scenario coverage generation must be deterministic for identical authored inputs.

## Runtime Evidence Mapping

- E2E summary evidence: `ops/e2e/generated/e2e-summary.json`
- Coverage evidence: `ops/e2e/generated/coverage-matrix.json`
- Curated drift evidence: `ops/_generated.example/docs-drift-report.json`

## Enforcement Links

- `checks_ops_domain_contract_structure`
- `checks_ops_required_files_contracts`
- `checks_ops_fixture_governance`
