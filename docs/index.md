---
title: bijux-atlas Documentation
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-21
---

# Bijux Atlas

`bijux-atlas` is a data-serving system that turns validated source inputs into
immutable release artifacts and serves those artifacts through stable query
surfaces.

## What Bijux Atlas Is

Atlas is not only a server and not only a build tool. It is one product that
covers:

- input validation and normalization
- deterministic artifact building
- catalog and store publication
- runtime query and operational serving

## Why Bijux Atlas Exists

Atlas exists to solve a common reliability problem: teams often mix source
inputs, intermediate files, and live runtime state into one mutable workflow.
That makes results hard to trust, hard to reproduce, and hard to operate.

Atlas keeps those boundaries explicit so you can answer practical questions
quickly:

- what exactly was built
- what exactly was published
- what exactly is being served
- what evidence supports promotion or rollback decisions

## What Bijux Atlas Does

```mermaid
flowchart LR
    source[Source Inputs] --> validate[Validation]
    validate --> build[Artifact Build]
    build --> publish[Catalog and Store Publish]
    publish --> serve[Runtime Serving]
```

This flow is the core of atlas behavior. The artifact and publication boundary
is deliberate: successful local processing alone is not treated as serving
truth until publication is complete.

## How Operations Fits In

Operations is a core part of atlas, not a side appendix. The operations surface
covers stack topology, Kubernetes rollout safety, observability, load budgets,
and release evidence.

When you run atlas in real environments, operations answers whether a change is
safe to install, promote, or roll back.

## Release Confidence Signals

Primary confidence and publication lanes:

- `repo/ci`
- `deploy-docs`
- `release-crates`
- `release-ghcr`
- `release-github`

These lanes are shown in the badges and are the main release health indicators
for atlas.

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

## Start Here

- product runtime and contracts: [Repository](bijux-atlas/index.md)
- deployment, observability, load, and release operations: [Operations](bijux-atlas-ops/index.md)
- governance and control-plane maintenance: [Maintainer](bijux-atlas-dev/index.md)

## Stability

This page is part of the canonical docs spine. Keep it aligned with active
runtime behavior, operations workflows, and release lanes.
