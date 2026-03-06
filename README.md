# Bijux Atlas

Deterministic genomics data infrastructure with executable governance.

## Product Narrative

| Surface | Location | Purpose |
| --- | --- | --- |
| Crates | `crates/` | Runtime, API, client SDK, ingest, query, benchmark, and control-plane binaries/libraries |
| Ops | `ops/` | Deploy, profile, schema, and evidence inputs |
| Docs | `docs/` | Canonical user/operator/contributor documentation |
| Docker | `docker/` | Container build and runtime image definitions |

## What's Real

Ten deterministic end-to-end runs are the core proof surface.

- Landing page: [`docs/tutorials/real-data/index.md`](docs/tutorials/real-data/index.md)
- Run catalog and intent: [`configs/tutorials/real-data-runs.json`](configs/tutorials/real-data-runs.json)
- Combined report: [`docs/reference/reports/real-data-runs.md`](docs/reference/reports/real-data-runs.md)
- Evidence dashboard: [`docs/tutorials/real-data/evidence-summary-dashboard.md`](docs/tutorials/real-data/evidence-summary-dashboard.md)
- E2E tutorial (commands + results): [`docs/tutorials/real-data-e2e.md`](docs/tutorials/real-data-e2e.md)
- E2E report page: [`docs/reference/reports/real-data-e2e-execution.md`](docs/reference/reports/real-data-e2e-execution.md)
- Reproducible workflow definition: [`configs/tutorials/real-data-runs-workflow.json`](configs/tutorials/real-data-runs-workflow.json)

## How To Evaluate

Run these checks as the fastest evaluator path:

1. `cargo run -p bijux-dev-atlas -- tutorials real-data list --format json`
2. `cargo run -p bijux-dev-atlas -- tutorials real-data plan --run-id rdr-001-genes-baseline --format json`
3. `cargo run -p bijux-dev-atlas -- tutorials run dataset-e2e --dataset-id genes-baseline --profile local --format json`
4. `cargo run -p bijux-dev-atlas -- tutorials run dataset-e2e --dataset-id genes-baseline --profile local --no-fetch --format json`
5. `cargo run -p bijux-dev-atlas -- tutorials real-data run-all --profile local --format json`

## Evidence First

Evidence is treated as a product contract, not an optional report.

- Human-readable summaries: `docs/_internal/generated/`
- Run-level artifacts: `artifacts/tutorials/runs/<run_id>/`
- Command execution log: `artifacts/tutorials/real-data-examples/command-log.json`
- Heavy-run verification report: `artifacts/tutorials/real-data-examples/check-results-heavy-partial.json`
- Contract and checks index: [`docs/_internal/governance/checks-and-contracts.md`](docs/_internal/governance/checks-and-contracts.md)

## Architecture Map

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

Release artifact references:

- Crates: `release/crates-v0.1.toml`
- Images: `release/images-v0.1.toml`
- Ops Helm OCI chart: `release/ops-v0.1.toml` (`oci://ghcr.io/bijux/charts/bijux-atlas`)
- Ops release manifest: `release/ops-release-manifest.json`
- Ops chart/workspace linkage manifest: `release/ops-release-bundle-manifest.json`

## Docs Deploy

- Workflow: [`.github/workflows/docs-deploy.yml`](.github/workflows/docs-deploy.yml)
- Build output: `artifacts/docs/site`
- Operator guide: [`docs/operations/docs-site-deploy.md`](docs/operations/docs-site-deploy.md)

## Runtime Image (GHCR)

- Image: `ghcr.io/<org>/bijux-atlas-runtime:<version>`
- Container policy: runtime image executes as `nonroot` user.
- Exposed port: `8080` (stable runtime HTTP port).

Run and verify health:

```bash
docker run --rm -p 8080:8080 ghcr.io/<org>/bijux-atlas-runtime:<version> atlas serve
curl -fsS http://127.0.0.1:8080/healthz
curl -fsS http://127.0.0.1:8080/readyz
```

## Tutorial Automation Migration

Tutorial automation is executed through `bijux-dev-atlas` commands. Legacy tutorial script entrypoints are removed. See [`docs/tutorials/run-with-dev-atlas.md`](docs/tutorials/run-with-dev-atlas.md).

## Perf Benchmark Runtime

CLI UX perf benchmarks are executed through `bijux-dev-atlas perf cli-ux bench` and `bijux-dev-atlas perf cli-ux diff`. The repository no longer keeps Python benchmark runners under `ops/`.

## Quick Start

### User

1. Start with [`docs/start-here.md`](docs/start-here.md).
2. Review API lifecycle in [`docs/api/lifecycle.md`](docs/api/lifecycle.md).
3. Use examples from [`docs/reference/examples/api-usage-examples.md`](docs/reference/examples/api-usage-examples.md).

### Operator

1. Read [`docs/operations/prerequisites.md`](docs/operations/prerequisites.md).
2. Follow [`docs/operations/deploy.md`](docs/operations/deploy.md).
3. Validate with operations readiness and observability guides.

### Contributor

1. Read [`docs/development/index.md`](docs/development/index.md).
2. Read [`docs/control-plane/index.md`](docs/control-plane/index.md).
3. Extend safely using [`docs/control-plane/extend-control-plane.md`](docs/control-plane/extend-control-plane.md).

## Repository Surfaces

- `crates/bijux-dev-atlas`: governance control-plane command surface
- `crates/bijux-atlas-server`: runtime service implementation
- `crates/bijux-atlas-client`: Rust client SDK crate for runtime consumers
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
Canonical structure rationale: [`docs/architecture/why-this-structure-exists.md`](docs/architecture/why-this-structure-exists.md)

## Version And Compatibility

![Version](https://img.shields.io/badge/version-workspace--managed-blue)
![Compatibility](https://img.shields.io/badge/compatibility-contract--governed-brightgreen)

Compatibility promise: [`docs/product/compatibility-promise.md`](docs/product/compatibility-promise.md)

## Documentation Entrypoints

![Docs](https://img.shields.io/badge/docs-github%20pages-blue)

Published by workflow: [`.github/workflows/docs-deploy.yml`](.github/workflows/docs-deploy.yml)

## Crate Versions

- `bijux-atlas-api`: workspace-managed version, API model surface
- `bijux-atlas-bench`: workspace-managed version, benchmark harness and perf scenarios
- `bijux-atlas-cli`: workspace-managed version, user CLI
- `bijux-atlas-client`: workspace-managed version, Rust client SDK
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

Security controls and operating rules are documented in [`docs/control-plane/security-posture.md`](docs/control-plane/security-posture.md) and linked operational guidance under [`docs/operations/security`](docs/operations/security).

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
