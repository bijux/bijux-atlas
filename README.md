# Bijux Atlas

Deterministic genomics data infrastructure with executable governance.

## What Exists

| Surface | Location | Purpose |
| --- | --- | --- |
| Crates | `crates/` | Runtime, API, ingest, query, and control-plane binaries/libraries |
| Ops | `ops/` | Deploy, profile, schema, and evidence inputs |
| Docs | `docs/` | Canonical user/operator/contributor documentation |
| Docker | `docker/` | Container build and runtime image definitions |

## Why This Is Different

- Determinism is enforced, not claimed.
- Governance is executable through checks/contracts.
- `bijux dev atlas` acts as SSOT control-plane for docs/configs/ops/release behavior.

## Evidence

- Contracts registry: [`docs/contract.md`](docs/contract.md)
- Checks and contracts index: [`docs/_internal/governance/checks-and-contracts.md`](docs/_internal/governance/checks-and-contracts.md)
- Generated docs contract coverage: [`docs/_internal/generated/docs-contract-coverage.json`](docs/_internal/generated/docs-contract-coverage.json)
- Generated docs quality dashboard: [`docs/_internal/generated/docs-quality-dashboard.json`](docs/_internal/generated/docs-quality-dashboard.json)

## Operational Maturity

Current supported maturity surfaces:

- Deterministic local and cluster deployment workflows
- Contract-validated documentation, configs, and policy surfaces
- Security and observability operational guidance with governed references
- Release evidence generation and validation pathways

## Not Yet Implemented

- Full artifact publishing automation for all distribution surfaces
- Final consolidation of long-tail duplicate narrative pages
- Complete public release orchestration for every crate/image/chart output

## Publishing Plan

Planned publication surfaces:

- `crates.io` for selected Rust crates
- `GHCR` for runtime and supporting container images
- `GitHub Pages` for docs site
- Versioned ops release artifacts for cluster operators

## Docs Deploy

- Workflow: [`.github/workflows/docs-deploy.yml`](.github/workflows/docs-deploy.yml)
- Build output: `artifacts/docs/site`
- Operator guide: [`docs/operations/docs-site-deploy.md`](docs/operations/docs-site-deploy.md)

## Quickstart By Persona

### User

1. Start with [`docs/start-here.md`](docs/start-here.md).
2. Review API lifecycle in [`docs/api/lifecycle.md`](docs/api/lifecycle.md).
3. Use examples from [`docs/reference/examples/index.md`](docs/reference/examples/index.md).

### Operator

1. Read [`docs/operations/prerequisites.md`](docs/operations/prerequisites.md).
2. Follow [`docs/operations/deploy.md`](docs/operations/deploy.md).
3. Validate with operations readiness and observability guides.

### Contributor

1. Read [`docs/development/index.md`](docs/development/index.md).
2. Read [`docs/control-plane/index.md`](docs/control-plane/index.md).
3. Extend safely using [`docs/control-plane/extend-control-plane.md`](docs/control-plane/extend-control-plane.md).

## Repo Map

- `crates/bijux-dev-atlas`: governance control-plane command surface
- `crates/bijux-atlas-server`: runtime service implementation
- `configs/`: policy/schema/configuration SSOTs
- `ops/`: deploy/release/validation operational surfaces
- `docs/`: canonical documentation plus internal generated evidence references

## Control-plane Is SSOT

```text
policies + schemas + docs contracts
          |
          v
   bijux dev atlas
          |
          v
generated artifacts + checks/contracts + publishable evidence
```

## Constitution

Repository constitution: [`CONTRACT.md`](CONTRACT.md)

## Next Reading

- Product narrative: [`docs/product/what-is-bijux-atlas.md`](docs/product/what-is-bijux-atlas.md)
- What we built: [`docs/product/what-we-built.md`](docs/product/what-we-built.md)
- Why trust this: [`docs/product/how-this-repo-enforces-itself.md`](docs/product/how-this-repo-enforces-itself.md)
