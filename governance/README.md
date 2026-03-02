# Governance Objects

Canonical governance object schema and generated object graph artifacts.

- Schema: `governance/object.json`
- Domain reviews: `governance/domain-review-dates.json`
- Registry mapping: `governance/domain-registry-map.json`
- Coverage baseline: `governance/coverage-baseline.json`
- List: `bijux dev atlas governance list --format json`
- Validate: `bijux dev atlas governance validate --format json`
- Artifacts:
  - `artifacts/governance/governance-graph.json`
  - `artifacts/governance/governance-summary.md`
  - `artifacts/governance/governance-coverage.json`
  - `artifacts/governance/governance-orphans.json`

## Cross-surface invariants

- Invariant: Helm env emitted by the chart must stay a subset of the runtime allowlist declared in `configs/contracts/env.schema.json`.
- Invariant: Every rollout profile under `ops/k8s/values/` must render successfully.
- Invariant: Every crate directory under `crates/` must be declared as a workspace member in the root `Cargo.toml`.
- Invariant: `mkdocs build --strict` must publish into the configured `site_dir`.
- Invariant: Docs must not contain references to missing pages.
- Invariant: No rollout profile may violate Helm chart fail guards.
- Invariant: Policy-surface configuration files must not be committed in minified form.
- Invariant: Unknown runtime `ATLAS_*` or `BIJUX_*` environment variables must fail startup unless the explicit local-dev override is enabled.
- Invariant: No single runtime behavior may be controlled through duplicate semantic environment variable names.
