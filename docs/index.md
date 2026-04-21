---
title: bijux-atlas Documentation
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-21
---

# Bijux Atlas

`bijux-atlas` is a Rust-first genomics dataset delivery platform.
It ingests governed GFF3 and FASTA inputs, builds immutable query artifacts,
publishes those artifacts into a serving catalog and store, and exposes stable
CLI and HTTP runtime surfaces.

<!-- bijux-atlas-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-atlas/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/ci.yml)
[![Docs](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-atlas/actions/workflows/release-crates.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-crates.yml)
[![GHCR Publish](https://github.com/bijux/bijux-atlas/actions/workflows/release-ghcr.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-ghcr.yml)
[![GitHub Release](https://github.com/bijux/bijux-atlas/actions/workflows/release-github.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-github.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-atlas?display_name=tag&label=release)](https://github.com/bijux/bijux-atlas/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-1%20package-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-atlas)
[![Published packages](https://img.shields.io/badge/published%20packages-1-2563EB)](https://github.com/bijux/bijux-atlas/tree/main/crates)

[![bijux-atlas](https://img.shields.io/crates/v/bijux-atlas?label=bijux--atlas&logo=rust)](https://crates.io/crates/bijux-atlas) [![bijux-atlas](https://img.shields.io/badge/bijux--atlas-ghcr-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)
[![bijux-atlas docs](https://img.shields.io/badge/docs-bijux--atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)
[![bijux-atlas rust-docs](https://img.shields.io/badge/rust--docs-bijux--atlas-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas/latest/bijux_atlas/)
[![Operations docs](https://img.shields.io/badge/docs-operations-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-ops/)
[![Maintainer docs](https://img.shields.io/badge/docs-maintainer-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-dev/)
<!-- bijux-atlas-badges:generated:end -->

## What It Does

Atlas combines four product responsibilities in one coherent workflow:

- validate and normalize source inputs
- build deterministic, immutable dataset artifacts
- publish release-shaped state to a serving store and catalog
- serve that state through query, API, and operational runtime surfaces

```mermaid
flowchart LR
    source[Governed source inputs] --> validate[Validation and normalization]
    validate --> build[Immutable artifact build]
    build --> publish[Catalog and store publish]
    publish --> serve[CLI and HTTP serving]
    serve --> observe[Operational evidence]
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

## What It Guarantees

- deterministic build behavior from governed inputs
- immutable release artifacts as the delivery unit
- explicit runtime, API, and config contracts
- operations and release evidence that can be reviewed and repeated

## What It Is Not

Atlas is not a generic mutable runtime that rewrites release truth in place.
It is not a replacement for source governance, and it is not a shortcut around
validation, publication, and release evidence.

## Operations Is Part of the Product

`bijux-atlas-ops` is not secondary documentation. It is where deployment,
rollout safety, observability, load budgets, and release trust are defined.

If your question is about running atlas safely in real environments, operations
is the primary handbook.

## Release Confidence Signals

Primary publication and confidence lanes:

- `repo/ci`
- `deploy-docs`
- `release-crates`
- `release-ghcr`
- `release-github`

These lanes are represented in the badges above and are the main release health
signals for atlas.

## Continue Reading

- runtime architecture, interfaces, workflows, and contracts: [Repository](bijux-atlas/index.md)
- deployment, rollout, observability, load, and release operations: [Operations](bijux-atlas-ops/index.md)
- governance, control-plane automation, and maintainer ownership: [Maintainer](bijux-atlas-dev/index.md)
