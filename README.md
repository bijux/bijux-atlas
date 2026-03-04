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

## Architecture

Repository architecture entrypoint: [`ARCHITECTURE.md`](ARCHITECTURE.md)

## Version And Compatibility

![Version](https://img.shields.io/badge/version-workspace--managed-blue)
![Compatibility](https://img.shields.io/badge/compatibility-contract--governed-brightgreen)

Compatibility promise: [`docs/product/compatibility-promise.md`](docs/product/compatibility-promise.md)

## Docs Site

![Docs](https://img.shields.io/badge/docs-github%20pages-blue)

Published by workflow: [`.github/workflows/docs-deploy.yml`](.github/workflows/docs-deploy.yml)

## Crate Versions

- `bijux-atlas-api`: workspace-managed version, API model surface
- `bijux-atlas-cli`: workspace-managed version, user CLI
- `bijux-atlas-core`: workspace-managed version, domain core
- `bijux-atlas-ingest`: workspace-managed version, ingest pipeline
- `bijux-atlas-model`: workspace-managed version, model semantics
- `bijux-atlas-policies`: workspace-managed version, policy engine
- `bijux-atlas-query`: workspace-managed version, query primitives
- `bijux-atlas-server`: workspace-managed version, runtime server
- `bijux-atlas-store`: workspace-managed version, storage abstractions
- `bijux-dev-atlas`: workspace-managed version, control-plane tooling

Crate details: [`docs/reference/crates.md`](docs/reference/crates.md)

## Crate Publishing Strategy

Current release plan keeps a multi-crate workspace with shared governance gates. We keep separate crates to preserve clear runtime boundaries and avoid collapsing public API, ops tooling, and control-plane concerns into a single package.

## Governance And Operations References

- Ops artifacts reference: [`docs/reference/ops.md`](docs/reference/ops.md)
- Docker reference: [`docs/reference/docker.md`](docs/reference/docker.md)
- Governance reference: [`docs/reference/governance.md`](docs/reference/governance.md)
- Release planning reference: [`docs/reference/release-plan.md`](docs/reference/release-plan.md)
- Crate release policy: [`docs/reference/crate-release-policy.md`](docs/reference/crate-release-policy.md)
- Contributor safety guide: [`docs/development/contributor-onboarding-rubric.md`](docs/development/contributor-onboarding-rubric.md)
- How to add checks and contracts: [`docs/control-plane/extend-control-plane.md`](docs/control-plane/extend-control-plane.md)

## Support Policy

Support is provided for governed, documented surfaces on `main` and release tags. Experimental surfaces may change with notice in docs and release notes.

## Security Posture

Security controls and operating rules are documented in [`docs/control-plane/security-posture.md`](docs/control-plane/security-posture.md) and linked operational runbooks under [`docs/operations/security`](docs/operations/security).

## Ops Reproducibility Posture

Release and ops reproducibility policy is governed by:

- [`configs/release/reproducibility-policy.json`](configs/release/reproducibility-policy.json)
- [`docs/operations/upgrade-compatibility-guide.md`](docs/operations/upgrade-compatibility-guide.md)
- `bijux dev atlas release reproducibility report`

## Institutional Readiness

Institutional release evidence and readiness inputs are documented in:

- [`docs/operations/institutional-packet.md`](docs/operations/institutional-packet.md)
- [`docs/operations/institutional-readiness-checklist.md`](docs/operations/institutional-readiness-checklist.md)

## Changelog Discipline

`CHANGELOG.md` structure and release note sections are contract-validated through `configs/release/version-policy.json` and `bijux dev atlas release validate`.

## Contact And Governance Owners

Primary ownership surfaces:

- Product and docs governance: `docs-governance`
- Runtime and release operations: `platform` and `bijux-atlas-operations`
- Control-plane governance tooling: `bijux-dev-atlas`

See ownership metadata in [`docs/_internal/governance/metadata/front-matter.index.json`](docs/_internal/governance/metadata/front-matter.index.json).

## Next Reading

- Product narrative: [`docs/product/what-is-bijux-atlas.md`](docs/product/what-is-bijux-atlas.md)
- What we built: [`docs/product/what-we-built.md`](docs/product/what-we-built.md)
- Why trust this: [`docs/product/how-this-repo-enforces-itself.md`](docs/product/how-this-repo-enforces-itself.md)
- Reliability boundaries: [`docs/product/reliability-boundaries.md`](docs/product/reliability-boundaries.md)
- Reviewer onboarding: [`docs/product/reviewer-onboarding-checklist.md`](docs/product/reviewer-onboarding-checklist.md)
