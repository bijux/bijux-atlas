# Bijux Atlas

<a id="top"></a>

**Bijux Atlas is a release-shaped genomics delivery system for turning governed
GFF3 and FASTA inputs into immutable query artifacts, stable APIs, and
auditable operational evidence.**

The workspace declares three installable command surfaces, eleven publishable
Rust crates, and one repository-only maintainer crate. Registry badges below
report live channel state; repository manifests define intended ownership and
must not be mistaken for proof that a particular release was published.

Declared commands:

* `bijux-atlas`, owned by `bijux-atlas-cli`,
* `bijux-atlas-server`, owned by `bijux-atlas-server`,
* `bijux-atlas-openapi`, owned by `bijux-atlas-api`.

Publishable crates:

* `bijux-atlas-core`: runtime-independent primitives and invariants,
* `bijux-atlas-model`: stable dataset and contract value types,
* `bijux-atlas-query`: query parsing, planning, cursoring, and execution,
* `bijux-atlas-ingest`: ingest normalization and artifact-build ownership,
* `bijux-atlas-store`: immutable publication and storage-backend contracts,
* `bijux-atlas-api`: API DTOs, OpenAPI ownership, and the `bijux-atlas-openapi` binary,
* `bijux-atlas-runtime`: canonical orchestration library composition,
* `bijux-atlas`: compatibility alias crate for the historical `bijux_atlas` Rust import path,
* `bijux-atlas-cli`: direct owner of the installed `bijux-atlas` command,
* `bijux-atlas-server`: direct owner of the installed `bijux-atlas-server` command,
* `bijux-atlas-ops`: operations-contract crate for stack, load, observability, and release support.

Repository-only crate:

* `bijux-atlas-dev`: maintainer control plane with `publish = false`.

The repeated `bijux-atlas` name is intentional but easy to misread. The
installed `bijux-atlas` command comes from `bijux-atlas-cli`, while the
`bijux-atlas` library crate is a separate compatibility alias.

Atlas is built around one public promise: the same release should describe what
was ingested, what was published, what can be queried, and what evidence
supports operating it. The repository exists to make those claims reviewable
instead of implicit.

<!-- bijux-atlas-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-atlas/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-atlas/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-atlas/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-crates.yml)
[![GHCR Publish](https://github.com/bijux/bijux-atlas/workflows/release-ghcr/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-ghcr.yml)
[![GitHub Release](https://github.com/bijux/bijux-atlas/workflows/release-github/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-github.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-atlas?display_name=tag&label=release)](https://github.com/bijux/bijux-atlas/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-11%20packages-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-atlas)
[![Published packages](https://img.shields.io/badge/published%20packages-11-2563EB)](https://github.com/bijux/bijux-atlas/tree/main/crates)

[![bijux-atlas](https://img.shields.io/crates/v/bijux-atlas?label=bijux--atlas&logo=rust)](https://crates.io/crates/bijux-atlas)
[![bijux-atlas-api](https://img.shields.io/crates/v/bijux-atlas-api?label=api&logo=rust)](https://crates.io/crates/bijux-atlas-api)
[![bijux-atlas-cli](https://img.shields.io/crates/v/bijux-atlas-cli?label=cli&logo=rust)](https://crates.io/crates/bijux-atlas-cli)
[![bijux-atlas-core](https://img.shields.io/crates/v/bijux-atlas-core?label=core&logo=rust)](https://crates.io/crates/bijux-atlas-core)
[![bijux-atlas-ingest](https://img.shields.io/crates/v/bijux-atlas-ingest?label=ingest&logo=rust)](https://crates.io/crates/bijux-atlas-ingest)
[![bijux-atlas-model](https://img.shields.io/crates/v/bijux-atlas-model?label=model&logo=rust)](https://crates.io/crates/bijux-atlas-model)
[![bijux-atlas-ops](https://img.shields.io/crates/v/bijux-atlas-ops?label=ops&logo=rust)](https://crates.io/crates/bijux-atlas-ops)
[![bijux-atlas-query](https://img.shields.io/crates/v/bijux-atlas-query?label=query&logo=rust)](https://crates.io/crates/bijux-atlas-query)
[![bijux-atlas-runtime](https://img.shields.io/crates/v/bijux-atlas-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-atlas-runtime)
[![bijux-atlas-server](https://img.shields.io/crates/v/bijux-atlas-server?label=server&logo=rust)](https://crates.io/crates/bijux-atlas-server)
[![bijux-atlas-store](https://img.shields.io/crates/v/bijux-atlas-store?label=store&logo=rust)](https://crates.io/crates/bijux-atlas-store)

[![ghcr-bijux--atlas](https://img.shields.io/badge/ghcr-bijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas)
[![ghcr-api](https://img.shields.io/badge/ghcr-api-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-api)
[![ghcr-cli](https://img.shields.io/badge/ghcr-cli-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-cli)
[![ghcr-core](https://img.shields.io/badge/ghcr-core-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-core)
[![ghcr-ingest](https://img.shields.io/badge/ghcr-ingest-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-ingest)
[![ghcr-model](https://img.shields.io/badge/ghcr-model-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-model)
[![ghcr-ops](https://img.shields.io/badge/ghcr-ops-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-ops)
[![ghcr-query](https://img.shields.io/badge/ghcr-query-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-query)
[![ghcr-runtime](https://img.shields.io/badge/ghcr-runtime-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-runtime)
[![ghcr-server](https://img.shields.io/badge/ghcr-server-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-server)
[![ghcr-store](https://img.shields.io/badge/ghcr-store-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-store)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)
[![bijux-atlas docs](https://img.shields.io/badge/docs-bijux--atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)
[![bijux-atlas rust-docs](https://img.shields.io/badge/rust--docs-bijux--atlas-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas/latest/bijux_atlas/)
[![Operations docs](https://img.shields.io/badge/docs-operations-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-ops/)
[![Maintainer docs](https://img.shields.io/badge/docs-maintainer-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-dev/)
<!-- bijux-atlas-badges:generated:end -->

Rust crate: [crates.io](https://crates.io/crates/bijux-atlas)
Rust API docs: [docs.rs](https://docs.rs/bijux-atlas/latest/bijux_atlas/)
Project docs: [bijux.io](https://bijux.io/bijux-atlas/)
Source docs spine: [`docs/index.md`](docs/index.md)

> **At a glance**
> Immutable datasets · Queryable release artifacts · Direct Cargo binaries ·
> Published split crates · Maintainer control plane · Governed docs, ops, and
> release evidence
> **Quality**
> Quality status is checked from live maintainer commands and checked-in contracts.
> `artifacts/` is disposable local output and is not part of the repository contract.
> Published artifacts and `v*` git tags define the public release line.
> Untagged checkout builds stay anchored to the latest real tag while the source tree can still be preparing the next release.

---

## Table of Contents

* [Why Atlas Exists](#why-atlas-exists)
* [Follow One Dataset End to End](#follow-one-dataset-end-to-end)
* [Declared Release Surface](#declared-release-surface)
* [How to Verify an Atlas Claim](#how-to-verify-an-atlas-claim)
* [Choose the Right Surface](#choose-the-right-surface)
* [How Atlas Fits With Bijux CLI](#how-atlas-fits-with-bijux-cli)
* [Key Features](#key-features)
* [Installation](#installation)
* [Verify Installed Surfaces](#verify-installed-surfaces)
* [Maintainer Control Plane](#maintainer-control-plane)
* [Packages, Configs, and Ops](#packages-configs-and-ops)
* [What Does Not Ship Yet](#what-does-not-ship-yet)
* [Project Tree](#project-tree)
* [Release Line & Stability](#release-line--stability)
* [Docs & Resources](#docs--resources)
* [Contributing](#contributing)
* [License](#license)

---

## Why Atlas Exists

Atlas is for dataset systems where a release is more important than a
long-running process and where the artifact boundary needs to stay visible.
It is a fit when:

* datasets must become immutable release artifacts,
* runtime APIs need explicit contracts and provenance,
* configs and ops inputs must be validated before they become policy,
* release claims should come from checked evidence instead of folklore,
* and maintainers need one honest control plane instead of scattered shell logic.

```mermaid
flowchart LR
    Sources[Governed source inputs] --> Validate[Validate and normalize]
    Validate --> Build[Build immutable artifacts]
    Build --> Verify[Verify artifact identity and integrity]
    Verify --> Store[Publish immutable store payload]
    Store --> Catalog[Promote dataset into catalog]
    Catalog --> Serve[Serve through CLI and HTTP]
    Serve --> Observe[Observe through metrics, logs, and contracts]
```

This is the center of gravity for the repository: Atlas does not primarily own
mutable runtime state. It owns the path from governed inputs to immutable
artifacts and then from immutable artifacts to stable delivery surfaces.

---

## Follow One Dataset End to End

The shortest useful evaluation is not a `--help` command or a health check. It
is the committed `tiny` dataset moving through every release boundary and then
answering an identity-bearing query:

| Boundary | Action | Result worth inspecting |
| --- | --- | --- |
| build | ingest the committed GFF3, FASTA, and FAI fixture | candidate manifest, SQLite payload, QC output, and exact dataset tuple |
| verify | validate structure, then run deep integrity verification | findings for the candidate bytes and references |
| publish | copy the verified payload into an immutable serving store | store lock, checksums, publication marker, and lifecycle record |
| promote | add the published tuple to the catalog | discoverable `release/species/assembly` identity |
| serve | start the server against the serving store, not the build directory | readiness, version, catalog, and query responses |

The [local Atlas walkthrough](docs/bijux-atlas/workflows/run-atlas-locally.md)
connects those boundaries without hiding them behind a bootstrap script. It
uses repository-owned fixtures and keeps disposable state under
`artifacts/getting-started/`. The walkthrough is evidence of the documented
local path only; it is not a production capacity, remote-store, security, or
failover qualification.

```mermaid
flowchart LR
    Fixture[Committed tiny fixture] --> Candidate[Candidate dataset]
    Candidate --> Verified[Verified artifact set]
    Verified --> Published[Immutable store payload]
    Published --> Catalog[Promoted catalog identity]
    Catalog --> Runtime[Ready runtime]
    Runtime --> Query[Identity-bearing query]
```

---

## Declared Release Surface

The workspace and release manifests declare:

* a runtime CLI named `bijux-atlas`, owned by `bijux-atlas-cli`,
* a server binary named `bijux-atlas-server`, owned by `bijux-atlas-server`,
* an OpenAPI export binary named `bijux-atlas-openapi`, owned by `bijux-atlas-api`,
* eleven publishable crates with explicit ownership split across core, model, query, ingest, store, API, runtime, compatibility, CLI, server, and operations contracts,
* one repository-only crate, `bijux-atlas-dev`, with `publish = false` for maintainer automation and release policy,
* governed `configs/`, `ops/`, `docs/`, and `makes/` trees that are validated together,
* and release inputs for crates, OCI release bundles, docs, and operations
  evidence.

```mermaid
flowchart TD
    Workspace[Repository workspace] --> Runtime[bijux-atlas-runtime crate]
    Workspace --> CLI[bijux-atlas-cli crate]
    Workspace --> ServerCrate[bijux-atlas-server crate]
    Workspace --> API[bijux-atlas-api crate]
    Workspace --> Alias[bijux-atlas alias crate]
    Workspace --> ControlPlane[bijux-atlas-dev crate]
    CLI --> RuntimeCLI[bijux-atlas]
    ServerCrate --> Server[bijux-atlas-server]
    API --> OpenAPI[bijux-atlas-openapi]
    Alias --> Compat[bijux_atlas import path]
    Workspace --> Docs[Numbered docs spine]
    Workspace --> Governance[configs and ops validation]
```

Runtime binaries, compatibility libraries, operational contracts, and
repository maintenance have separate owners and release obligations.

## How to Verify an Atlas Claim

Atlas distinguishes a declared contract from evidence that the contract held
for a particular build, deployment, or release. Source code and schemas define
what must be true; generated references make those rules inspectable; run
reports show what was exercised; signed release material binds the result to
the artifacts consumers receive.

```mermaid
flowchart LR
    Contract[Code, schema, and policy] --> Reference[Generated reference]
    Reference --> Execution[Named validation or scenario]
    Execution --> Report[Machine-readable report]
    Report --> Binding[Checksum and provenance binding]
    Binding --> Decision[Consumer-verifiable decision]
```

| Claim | Primary authority | Evidence required for a concrete release |
| --- | --- | --- |
| a command or HTTP shape is supported | owning crate plus generated CLI or OpenAPI reference | contract validation tied to the source revision |
| a dataset is publishable | ingest, artifact, and store contracts | manifest, hashes, provenance, and store publication record |
| a dataset is discoverable | catalog identity and promotion contracts | promoted catalog entry bound to the published payload |
| a deployment is admissible | chart schema, profile values, and Kubernetes policy | render and admission evidence plus results from the exact executable checks selected for the profile |
| a performance budget holds | named scenario, threshold, and metric contract | measured run plus baseline comparison from the same scenario identity |
| a release is distributable | channel manifests and signing policy | coherent packet, checksums, provenance, and verifier result |

A checked-in sample, schema-valid fixture, or empty report inventory proves
shape only. It does not prove that a live scenario ran or that the current
release passed. The [operations handbook](docs/bijux-atlas-ops/index.md)
separates these evidence levels for deployment and promotion decisions.

## Choose the Right Surface

Start with the surface that matches the job in front of you:

| If you need... | Start here | Why |
| --- | --- | --- |
| the end-user Atlas command | `bijux-atlas-cli` | owns the installed `bijux-atlas` binary |
| the long-running HTTP process | `bijux-atlas-server` | owns server startup, telemetry bootstrap, and route exposure |
| OpenAPI export and API wire contracts | `bijux-atlas-api` | owns DTOs, parameters, envelopes, and `bijux-atlas-openapi` |
| the orchestration library | `bijux-atlas-runtime` | composes ingest, query, store, API, and runtime policy |
| the historical Rust import path | `bijux-atlas` | preserves the `bijux_atlas` compatibility surface |
| stack, load, and observability contracts | `bijux-atlas-ops` | owns operator-facing reference and release-support surfaces |
| maintainer automation and repository law | `bijux-atlas-dev` | owns governance, docs validation, release planning, and reports |

### Release Surface Directory

| Surface | Ownership | Release contract |
| --- | --- | --- |
| `bijux-atlas` | `bijux-atlas-cli` | direct end-user CLI binary |
| `bijux-atlas-server` | `bijux-atlas-server` | direct HTTP runtime binary |
| `bijux-atlas-openapi` | `bijux-atlas-api` | direct OpenAPI export binary |
| `bijux-atlas-runtime` | `bijux-atlas-runtime` | canonical Rust orchestration crate |
| `bijux-atlas` crate | `crates/bijux-atlas/` | compatibility alias for the historical `bijux_atlas` Rust import path |
| `bijux-atlas-ops` | `bijux-atlas-ops` | operations-contract crate |
| `bijux-atlas-dev` | `bijux-atlas-dev` | repository-only maintainer control plane with `publish = false` |

## Crate Boundary Map

Atlas currently enforces a twelve-crate workspace boundary:

* `crates/bijux-atlas-core/`: runtime-independent primitives and invariants
* `crates/bijux-atlas-model/`: stable dataset, gene, diff, and policy types
* `crates/bijux-atlas-query/`: query parsing, planning, cursoring, and SQLite execution
* `crates/bijux-atlas-ingest/`: ingest normalization, anomaly handling, and artifact build execution
* `crates/bijux-atlas-store/`: immutable artifact publication and backend contracts
* `crates/bijux-atlas-api/`: API DTOs, Rust client surface, OpenAPI generation, and the `bijux-atlas-openapi` binary
* `crates/bijux-atlas-runtime/`: canonical runtime composition crate with orchestration, policies, runtime config, and cache wiring
* `crates/bijux-atlas/`: compatibility alias crate for the historical `bijux_atlas` Rust import path
* `crates/bijux-atlas-cli/`: direct `bijux-atlas` binary owner plus CLI contract tests
* `crates/bijux-atlas-server/`: direct `bijux-atlas-server` binary owner plus server-facing integration tests and benchmarks
* `crates/bijux-atlas-ops/`: repository-owned operational path, Kubernetes, load, and release-support contracts
* `crates/bijux-atlas-dev/`: maintainer-only control plane for governance and repository operations

The boundary contract for this map lives at
[`docs/bijux-atlas/foundations/crate-boundary-contract.md`](docs/bijux-atlas/foundations/crate-boundary-contract.md).

---

## How Atlas Fits With Bijux CLI

Atlas owns the genomic dataset runtime and release model.
`bijux-cli` owns the umbrella command runtime that can host Atlas alongside
other Bijux tools.

Choose one command identity per environment:

* use `bijux-atlas`, `bijux-atlas-server`, and `bijux-atlas-openapi` when you want the Atlas binaries directly
* use `bijux atlas ...` and `bijux dev atlas ...` when you already standardize on the `bijux` umbrella runtime

The routed and direct entrypoints should describe the same Atlas runtime
surface. The difference is packaging and command routing, not a different
product contract.

---

## Key Features

### Immutable Dataset Delivery

Atlas treats dataset builds as release artifacts with explicit manifests, provenance, and reproducible packaging inputs.

### Split Runtime Surfaces With Clear Ownership

`bijux-atlas` is owned by `bijux-atlas-cli`.
`bijux-atlas-server` is owned by `bijux-atlas-server`.
`bijux-atlas-openapi` remains a user-facing Atlas binary, but it is API-owned and built from `bijux-atlas-api`.
The installed umbrella runtime namespace is `bijux atlas ...`.
The maintainer namespace is `bijux dev atlas ...`, backed by the `bijux-atlas-dev` binary.

### Governed Repository Inputs

`configs/`, `ops/`, `docs/`, and `makes/` are checked together so release, policy, and operating guidance can stay aligned with the code that uses them.

### Thin Makes Wrapper Layer

GNU Make exists as a boring convenience layer rooted at [`makes/root.mk`](makes/root.mk).
Orchestration logic belongs in Rust commands, not in shell-heavy wrapper files.

### Honest Release Evidence

The release story includes checked manifests, compatibility tables, docs deployment, crates.io publication, and GitHub release automation instead of one-off manual steps.

### Operations as a Governed System

The `ops/` contract system is not a collection of deployment examples. It owns
profile intent, Helm and Kubernetes contracts, network and workload security,
service topology, observability packs, load scenarios and thresholds, drift
detection, recovery drills, checksums, provenance, and release evidence.
Reusable Rust models live in `bijux-atlas-ops`; executable repository
orchestration belongs to `bijux-atlas-dev`.

```mermaid
flowchart LR
    Profile[Environment profile] --> Render[Render and validate]
    Render --> Deploy[Deploy published artifacts]
    Deploy --> Signals[Health, metrics, logs, traces]
    Signals --> Stress[Load and failure evidence]
    Stress --> Release[Promotion or rollback decision]
    Release --> Proof[Checksums, provenance, and release packet]
```

The operational surface is organized by decisions rather than tool names:

| Decision | Start with | Completion evidence |
| --- | --- | --- |
| admit or remove traffic | [Health, Readiness, and Drain](docs/bijux-atlas-ops/observability/health-readiness-and-drain.md) | stable probe window plus representative user-path behavior |
| diagnose a request | [Logging, Metrics, and Tracing](docs/bijux-atlas-ops/observability/logging-metrics-and-tracing.md) | correlated request ID, trace, logs, and population metrics |
| isolate data or cache failure | [Cache and Store Operations](docs/bijux-atlas-ops/stack/cache-and-store-operations.md) | verified store authority and bounded cache recovery |
| publish a release | [Distribution Channels](docs/bijux-atlas-ops/release/distribution-channels.md) | required immutable channel identities resolve and agree |
| accept transported evidence | [Release Packets](docs/bijux-atlas-ops/release/release-packets.md) | fresh consumer verification receipt |

A successful command, healthy process, uploaded artifact, or present report is
only one checkpoint. Promotion requires the identities and evidence across the
relevant decision path to agree.

---

## Installation

Use direct Cargo installation when you want Atlas by itself, or when CI and
local Rust workflows call the binaries directly. For a version available on
crates.io, install the direct binaries with:

```bash
cargo install --locked bijux-atlas-cli --bin bijux-atlas
cargo install --locked bijux-atlas-server --bin bijux-atlas-server
cargo install --locked bijux-atlas-api --bin bijux-atlas-openapi
bijux-atlas --help
bijux-atlas version
```

If you already operate through the sibling `bijux-cli` umbrella runtime, the
same Atlas surfaces can also be reached as `bijux atlas ...` and `bijux dev
atlas ...`. That routed command story is secondary to the direct Cargo-managed
Atlas binaries documented here.

Publishable crates are intended to be consumed directly from Cargo. Atlas
does not hide the release line behind a repository-only bootstrap wrapper.

Quick verification for the standalone binaries:

```bash
bijux-atlas version
bijux-atlas --help
bijux-atlas-server --help
bijux-atlas-openapi --help
```

From a workspace checkout, run the current source tree directly with:

```bash
cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- version
cargo run -q -p bijux-atlas-server --bin bijux-atlas-server -- --help
cargo run -q -p bijux-atlas-api --bin bijux-atlas-openapi -- --help
cargo run -q -p bijux-atlas-dev -- --help
```

Runtime crates are configured for Cargo publication. The maintainer crate is
part of the repository contract and the `bijux dev atlas ...` umbrella
surface, even when you run it directly from a checkout.

Atlas does not publish a Python package yet. The planned Python bridge is a future release item, not a hidden install path today.

---

## Verify Installed Surfaces

```bash
# Inspect the runtime surface
bijux-atlas --help
bijux-atlas version

# Export the OpenAPI document
bijux-atlas-openapi --help

# Inspect the server surface
bijux-atlas-server --help
```

For the canonical runtime references, start with:

* [`docs/bijux-atlas/workflows/index.md`](docs/bijux-atlas/workflows/index.md)
* [`docs/bijux-atlas-ops/index.md`](docs/bijux-atlas-ops/index.md)
* [`docs/bijux-atlas/interfaces/command-surface.md`](docs/bijux-atlas/interfaces/command-surface.md)

```mermaid
flowchart LR
    Inspect[Inspect CLI and server] --> Install[Confirm binary identity]
    Install --> BuildDocs[Follow getting started]
    BuildDocs --> Ingest[Build and verify a dataset]
    Ingest --> Publish[Publish payload and promote catalog]
    Publish --> Serve[Start the server]
    Serve --> Query[Run identity-bearing queries]
```

The commands above confirm binary ownership and product shape. They do not boot
a server or prove dataset behavior. Follow the workflow guide for those claims,
and keep the binary, dataset, store, and catalog identities visible throughout.

---

## Maintainer Control Plane

Atlas keeps repository automation explicit:

```bash
bijux dev atlas --help
cargo run -q -p bijux-atlas-dev -- --help
make help
```

Use `bijux dev atlas ...` as the canonical installed automation surface.
Use `bijux-atlas-dev` or `cargo run -p bijux-atlas-dev -- ...` when you are working from a checkout.
Use `make` only through the curated wrappers exposed from [`makes/root.mk`](makes/root.mk).

Helpful maintainer entrypoints:

```bash
cargo run -q -p bijux-atlas-dev -- docs doctor --format json
cargo run -q -p bijux-atlas-dev -- governance validate --format json
cargo run -q -p bijux-atlas-dev -- release validate --format json
make ci-fast
```

```mermaid
flowchart LR
    Change[Contributor change] --> Validate[Governance and doctor checks]
    Validate --> Test[Test and evidence]
    Test --> Release[Release validation and generation]
```

The maintainer surface is separate on purpose. It keeps repository validation, docs generation, and
release evidence out of the runtime binaries that end users depend on.

---

## Packages, Configs, and Ops

Atlas carries more release-facing material in-repo than a typical single-crate project.
That is intentional, but the boundaries stay explicit:

* `crates/` owns the publishable Atlas crate set plus the repository-only maintainer control plane crate,
* `configs/` owns policy, schema, registry, and repository inputs,
* `ops/` owns deployment, observability, release, and scenario data,
* `docs/` owns the package handbooks and contract references,
* `makes/` owns the thin wrapper surface over governed commands.

The goal is not “everything is public API.”
The goal is one honest source of truth for each governed concern.

```mermaid
flowchart TD
    Repo[Repository] --> Crates[crates]
    Repo --> Configs[configs]
    Repo --> Ops[ops]
    Repo --> Docs[docs]
    Repo --> Makes[makes]
    Docs --> Handbook[Product and operations handbook]
    Configs --> Policy[Policy and schema sources]
    Ops --> RuntimeOps[Operational inputs]
```

This repository map helps explain why Atlas looks broader than a single-crate project. The extra
trees are not incidental clutter; they are part of the governed release and operations surface.

---

## What Does Not Ship Yet

Atlas is deliberately explicit about non-shipped scope:

* there is no published `bijux-atlas-python` package yet,
* there is no mutable lab workflow engine inside the runtime,
* and `artifacts/` is not a source-of-truth tree.

If a surface is planned, internal, or future-facing, it should be described as such instead of being implied by README language.

---

## Project Tree

| Path | Purpose |
| --- | --- |
| `crates/bijux-atlas/` | Compatibility alias crate for the historical `bijux_atlas` import path |
| `crates/bijux-atlas-runtime/` | Canonical runtime orchestration crate |
| `crates/bijux-atlas-dev/` | Repository-only maintainer control plane for docs, checks, governance, release, configs, and ops |
| `crates/bijux-atlas-ops/` | Published operational contract and stack-support crate |
| `configs/` | Repository-owned policy, schemas, registries, and examples |
| `ops/` | Release specs, scenarios, deployment inputs, observability, and contracts |
| `makes/` | Thin GNU Make wrapper surface |
| `docs/` | Product, operations, and maintainer handbook |
| `artifacts/` | Generated local outputs and evidence |

---

## Release Line & Stability

Crates.io records, GitHub releases, deployed documentation, and `v*` git tags
define the observable public release line.
Untagged checkout builds derive their operator-facing version from the latest
real tag. Workspace manifests and checked-in release inputs can move ahead for
the next intended release.
The workspace version and release inputs express intent, not channel state.
Confirm availability through the live registry and release badges before
installing or promoting a version. `bijux-atlas-dev` remains repository-only,
and this repository does not declare a Python publication channel.

Release expectations live in [`docs/bijux-atlas-dev/delivery/release-and-versioning.md`](docs/bijux-atlas-dev/delivery/release-and-versioning.md).
Badge contract expectations live in [`docs/bijux-atlas-dev/governance/badge-catalog.md`](docs/bijux-atlas-dev/governance/badge-catalog.md).
Compatibility and operational promises live under [`docs/bijux-atlas/contracts/index.md`](docs/bijux-atlas/contracts/index.md).

---

## Docs & Resources

Start with the package handbooks:

* product runtime: [`docs/bijux-atlas/index.md`](docs/bijux-atlas/index.md)
* operations: [`docs/bijux-atlas-ops/index.md`](docs/bijux-atlas-ops/index.md)
* maintainer control plane: [`docs/bijux-atlas-dev/index.md`](docs/bijux-atlas-dev/index.md)

Root policies:

* contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
* security policy: [`SECURITY.md`](SECURITY.md)
* code ownership: [`.github/CODEOWNERS`](.github/CODEOWNERS)

---

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md).
Use small, coherent Conventional Commit / Commitizen-style commits such as `fix(configs): ...` or `refactor(makes): ...`.

---

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
