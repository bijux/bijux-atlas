# Contract

- Area: `ops/e2e`
- schema_version: `1`
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

## Invariants

- E2E is composition-only: scenarios orchestrate existing stack, load, observe, and dataset contracts without redefining them.
- Each suite entry references existing scenarios and expectations by stable identifiers.
- Reproducibility settings in `reproducibility-policy.json` are mandatory for every promoted suite.
- Fixtures referenced by E2E scenarios must be present and allowlisted before execution.
- Generated summary and coverage artifacts are deterministic for the same suite input and fixture set.
- Scenario taxonomy must classify all suites and scenarios used in the promoted E2E catalog.
