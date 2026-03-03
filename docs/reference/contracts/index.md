# Contracts

- Owner: `docs-governance`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: provide one canonical map of machine-readable contracts and their human references.

## Contract map

- API surfaces: [Endpoints](endpoints.md), [Errors](errors.md)
- Health semantics: [Health Endpoints](health-endpoints.md)
- Observability: [Observability Contracts](observability.md)
- Runtime configuration: [Config Keys](config-keys.md), [Chart Values](chart-values.md)
- Ops governance: [Helm Env Allowlist Subset](ops/helm-env-subset.md), [Profile Matrix](ops/profile-matrix.md)
- Release evidence: [Release Evidence Contracts](release/evidence.md)
- Docs pipeline: [Site Output](docs/site-output.md)
- Data and quality: [Artifact contracts](artifacts/index.md), [QC](qc.md)
- Telemetry: [Telemetry](telemetry.md)
- Plugin contracts: [Plugins](plugins/index.md)
- Schema files: [Schemas Index](schemas/index.md)
- Minimal examples: [Examples](examples/index.md)

## Rules

- Contract facts live in `contracts/schemas/*.json`.
- Narrative guides must link here instead of embedding full schema content.
- This index is the only reader-facing contract portal.

## Next steps

- [Reference Index](../index.md)
- [API](../../api/index.md)
