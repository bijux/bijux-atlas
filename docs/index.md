---
title: bijux-atlas Documentation
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-25
---

# Bijux Atlas

`bijux-atlas` is a governed release-backed data product for genomics datasets.
It takes governed GFF3 and FASTA inputs through explicit validation and
normalization, builds deterministic release artifacts, publishes those artifacts
into serving state, and exposes them through stable CLI, HTTP, and operational
surfaces.

Atlas exists to convert raw domain inputs into governed, release-backed,
trustworthy serving state.

<!-- bijux-atlas-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-atlas/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-atlas/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-atlas/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-crates.yml)
[![GHCR Publish](https://github.com/bijux/bijux-atlas/workflows/release-ghcr/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-ghcr.yml)
[![GitHub Release](https://github.com/bijux/bijux-atlas/workflows/release-github/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-github.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-atlas?display_name=tag&label=release)](https://github.com/bijux/bijux-atlas/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-1%20package-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-atlas)
[![Published packages](https://img.shields.io/badge/published%20packages-1-2563EB)](https://github.com/bijux/bijux-atlas/tree/main/crates)

[![bijux-atlas](https://img.shields.io/crates/v/bijux-atlas?label=bijux--atlas&logo=rust)](https://crates.io/crates/bijux-atlas)
[![bijux-atlas](https://img.shields.io/badge/bijux--atlas-ghcr-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)
[![bijux-atlas docs](https://img.shields.io/badge/docs-bijux--atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)
[![bijux-atlas rust-docs](https://img.shields.io/badge/rust--docs-bijux--atlas-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas/latest/bijux_atlas/)
[![Operations docs](https://img.shields.io/badge/docs-operations-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-ops/)
[![Maintainer docs](https://img.shields.io/badge/docs-maintainer-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-dev/)
<!-- bijux-atlas-badges:generated:end -->

## What Atlas Actually Is

Atlas is not only a server and not only a CLI. It is a full system for
building, publishing, serving, operating, and evolving release-shaped data
without hiding the artifact boundary behind mutable runtime behavior.

The center of gravity is the release artifact, not the running process. That is
why Atlas keeps ingest, build, publication, serving, and operational evidence
as explicit surfaces instead of letting them blur together.

Atlas combines four product responsibilities in one coherent path:

- validate and normalize source inputs
- build deterministic and immutable dataset artifacts
- publish release-backed state to a serving store and catalog
- serve that state through query, API, and operational runtime surfaces

```mermaid
flowchart LR
    source[Governed GFF3 and FASTA inputs] --> validate[Validation and normalization]
    validate --> build[Deterministic artifact build]
    build --> release[Immutable release artifacts]
    release --> publish[Catalog and store publish]
    publish --> serve[CLI and HTTP runtime surfaces]
    serve --> observe[Observability and release evidence]
```

## Why It Exists

Atlas exists to avoid a common failure mode in data systems: mixing raw inputs,
intermediate files, and mutable runtime state into one opaque process.

Atlas keeps those boundaries explicit so teams can answer high-stakes questions
without guessing:

- what was actually built
- what was actually published
- what is currently served
- what evidence supports promotion, rollback, or incident decisions

Atlas is strongest when teams need trusted serving of governed release data
rather than a convenient but opaque runtime that quietly mutates its own truth.

```mermaid
flowchart LR
    subgraph avoid[What Atlas avoids]
        raw[Raw inputs]
        intermediate[Intermediate files]
        mutable[Mutable runtime truth]
        opaque[Opaque operational decisions]
        raw --- intermediate
        intermediate --- mutable
        mutable --- opaque
    end

    avoid --> boundary

    subgraph boundary[What Atlas enforces]
        governed[Governed inputs]
        validated[Validated build boundary]
        immutable[Immutable release artifacts]
        serving[Published serving state]
        evidence[Operational and release evidence]
        governed --> validated --> immutable --> serving --> evidence
    end
```

## What It Guarantees

- deterministic build behavior from governed inputs
- immutable release artifacts as the delivery unit
- explicit runtime, API, and configuration contracts
- release and operations evidence that can be reviewed and repeated

## What It Is Not

Atlas is not a generic mutable runtime that rewrites release truth in place.
It is not a replacement for source governance, and it is not a shortcut around
validation, publication, and release evidence.

## Atlas Has Four Linked Concerns

Atlas is easier to understand when its main concerns are explicit instead of
collapsed into one generic idea of "the runtime".

```mermaid
flowchart TB
    atlas[Bijux Atlas]

    atlas --> runtime[Runtime and product]
    atlas --> maintainer[Maintainer control plane]
    atlas --> ops[Operations]
    atlas --> trust[Security and trust]

    runtime --> runtime_a[Datasets and releases]
    runtime --> runtime_b[CLI, HTTP, and OpenAPI surfaces]
    runtime --> runtime_c[Runtime contracts]

    maintainer --> maintainer_a[Ownership and workflow control]
    maintainer --> maintainer_b[Automation and governance]
    maintainer --> maintainer_c[Delivery and compatibility]

    ops --> ops_a[Deployment and stack]
    ops --> ops_b[Rollout safety and recovery]
    ops --> ops_c[Observability and load]

    trust --> trust_a[Provenance and reproducibility]
    trust --> trust_b[Policy enforcement and drift control]
    trust --> trust_c[Release confidence and safe change]
```

### Runtime and Product

This is the product face readers usually mean when they say "Atlas":
datasets, releases, queries, interfaces, and contracts.

### Maintainer Control Plane

Atlas is not meant to be changed through informal repository habits. The
maintainer surface exists so ownership, workflow control, automation, and
compatibility policy stay explicit instead of tribal.

### Operations

Atlas is a real deployed system, not just a local Rust binary. Deployment,
rollout safety, observability, load, recovery, and release evidence are part of
the product model.

### Security and Trust

Trust is not only vulnerability scanning. It covers provenance,
reproducibility, drift detection, controlled exceptions, and whether a release
can actually be believed.

## Operations and Trust Are Part of the Product

`bijux-atlas-ops` is not secondary documentation. It is where deployment,
rollout safety, observability, load budgets, and release trust are defined.

If your question is about running Atlas safely in real environments, operations
is the primary handbook.

The same is true of trust. Security and release assurance are not side checks
after the runtime is done. They are part of how Atlas proves what was built,
what was promoted, and what should be rolled back.

## Release Confidence Signals

Primary publication and confidence lanes:

- `repo/ci`
- `deploy-docs`
- `release-crates`
- `release-ghcr`
- `release-github`

These lanes are represented in the badges above, but the important point is not
the badges themselves. Atlas uses them to decide whether a release is ready to
promote, hold, or roll back.

```mermaid
flowchart TB
    source[Source changes] --> ci[repo and ci]
    ci --> docs[Docs and contract visibility]
    docs --> package[Release packaging]
    package --> crates[release-crates]
    package --> ghcr[release-ghcr]
    package --> github[release-github]
    crates --> confidence[Confidence signals]
    ghcr --> confidence
    github --> confidence
    confidence --> decisions[Promotion, rollback, and incident decisions]
```

Atlas is not complete when it merely builds. It is complete when build, docs,
contracts, publication channels, and operational evidence line up tightly
enough that release decisions are reviewable instead of improvised.

## Start From the Right Handbook

The three handbook surfaces are separated on purpose because they answer
different classes of questions.

### Repository

Use [Repository](bijux-atlas/index.md) when the question is about the Atlas
product itself: datasets, releases, workflows, interfaces, runtime
architecture, and compatibility contracts.

### Operations

Use [Operations](bijux-atlas-ops/index.md) when the question is about how Atlas
runs safely: deployment, rollout safety, observability, load, recovery, and
release operations.

### Maintainer

Use [Maintainer](bijux-atlas-dev/index.md) when the question is about how Atlas
changes safely: ownership, automation, workflow control, delivery, and
governance.

### Read Next

- product model and core boundaries: [What Atlas Is](bijux-atlas/foundations/what-atlas-is.md)
- runtime architecture, interfaces, workflows, and contracts:
  [Repository](bijux-atlas/index.md)
- deployment, rollout, observability, load, and release operations:
  [Operations](bijux-atlas-ops/index.md)
- governance, control-plane automation, and maintainer ownership:
  [Maintainer](bijux-atlas-dev/index.md)

## Purpose

This page explains Atlas as a whole system before readers dive into the
repository, operations, or maintainer handbooks. It is the high-level contract
for what Atlas is for, why its boundaries exist, and how the major handbook
surfaces fit together.

## Stability

This page is part of the canonical docs spine. Keep it aligned with the current
Atlas release model, runtime surfaces, operations surface, and maintainer
control plane.
