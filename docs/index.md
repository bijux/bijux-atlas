---
title: bijux-atlas Documentation
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-21
---

# Bijux Atlas

<section class="bijux-hero">
  <div class="bijux-hero__eyebrow">Genomics Query and Operations Platform</div>
  <h1 class="bijux-hero__title">Operate a deterministic genomics API stack with explicit runtime, operations, and maintainer boundaries.</h1>
  <p class="bijux-hero__lede">`bijux-atlas` is organized as three connected handbooks so you can move from API surface and runtime contracts, to operating procedures, to repository control-plane governance without losing context.</p>
  <div class="bijux-topic-row">
    <span class="bijux-topic-pill">HTTP query contracts</span>
    <span class="bijux-topic-pill">Ops and release surfaces</span>
    <span class="bijux-topic-pill">Governance and policies</span>
    <span class="bijux-topic-pill">Reproducible maintenance</span>
  </div>
</section>

<div class="bijux-callout">
  <strong>Start from the pressure you have right now.</strong> Use <em>Repository</em> for runtime/API behavior, <em>Operations</em> for deployment and observability workflows, and <em>Maintainer</em> for policy, automation, and ownership rules.
</div>

<!-- bijux-atlas-badges:generated:start -->
[![Rust 1.86.0](https://img.shields.io/badge/rust-1.86.0-000000?logo=rust)](https://github.com/bijux/bijux-atlas/blob/main/rust-toolchain.toml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![CI PR](https://github.com/bijux/bijux-atlas/actions/workflows/ci-pr.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/ci-pr.yml)
[![Docs](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml)
[![Release Candidate](https://github.com/bijux/bijux-atlas/actions/workflows/release-candidate.yml/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-candidate.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-atlas?display_name=tag&label=release)](https://github.com/bijux/bijux-atlas/releases)
[![Published crate](https://img.shields.io/badge/published%20crate-1-2563EB)](https://crates.io/crates/bijux-atlas)

[![bijux-atlas](https://img.shields.io/crates/v/bijux-atlas?label=bijux--atlas&logo=rust)](https://crates.io/crates/bijux-atlas)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)
[![Operations docs](https://img.shields.io/badge/docs-operations-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-ops/)
[![Maintainer docs](https://img.shields.io/badge/docs-maintainer-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-dev/)
<!-- bijux-atlas-badges:generated:end -->

<div class="bijux-panel-grid">
  <div class="bijux-panel">
    <h3>Repository Handbook</h3>
    <p>Runtime package boundaries, HTTP interfaces, architecture seams, query behavior, contracts, and quality gates for `bijux-atlas`.</p>
  </div>
  <div class="bijux-panel">
    <h3>Operations Handbook</h3>
    <p>Stack lifecycle, Kubernetes surfaces, release procedures, observability, incident workflows, and load/performance verification.</p>
  </div>
  <div class="bijux-panel">
    <h3>Maintainer Handbook</h3>
    <p>Control-plane ownership for check suites, policy enforcement, governance contracts, repo standards, and automation maintenance.</p>
  </div>
</div>

<div class="bijux-quicklinks">
  <a class="md-button md-button--primary" href="bijux-atlas/">Open Repository Handbook</a>
  <a class="md-button" href="bijux-atlas-ops/">Open Operations Handbook</a>
  <a class="md-button" href="bijux-atlas-dev/">Open Maintainer Handbook</a>
</div>

## Start Paths

Use the path that matches your immediate decision:

- API/runtime behavior, query semantics, and interface contracts:
  [Repository](bijux-atlas/index.md)
- deployment, stack operations, and operational readiness:
  [Operations](bijux-atlas-ops/index.md)
- policy ownership, governance contracts, and maintenance workflows:
  [Maintainer](bijux-atlas-dev/index.md)

## How The Documentation Is Structured

```mermaid
flowchart LR
    Home[Atlas Home] --> Repo[Repository Handbook]
    Home --> Ops[Operations Handbook]
    Home --> Maint[Maintainer Handbook]
    Repo --> Ops
    Ops --> Maint
    Maint --> Repo
```

The three handbooks are deliberately separate so operational guidance does not get mixed into runtime docs, and maintainer governance does not get buried inside product-facing pages.

## System Orientation

```mermaid
flowchart TD
    Client[API clients] --> Runtime[bijux-atlas runtime]
    Runtime --> Store[Artifact and metadata stores]
    Runtime --> Observe[Metrics and tracing]
    Observe --> Ops[Operational playbooks]
    RepoGov[Maintainer governance] --> Runtime
    RepoGov --> Ops
```

Use this map when you need to reason about where a change belongs:

- runtime behavior and interface guarantees belong in `bijux-atlas`
- operating procedures and production controls belong in `bijux-atlas-ops`
- automation and policy ownership belong in `bijux-atlas-dev`

## Package Handbooks

- [Repository](bijux-atlas/index.md)
- [Operations](bijux-atlas-ops/index.md)
- [Maintainer](bijux-atlas-dev/index.md)

## Purpose

This home page is the stable orientation layer for the entire `bijux-atlas` documentation set. Keep it explicit enough that a reader returning much later can immediately choose the correct handbook without re-learning repository internals.
