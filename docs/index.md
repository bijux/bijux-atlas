---
title: bijux-atlas Documentation
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-21
---

# Bijux Atlas

`bijux-atlas` is a governed atlas runtime with a first-class operations surface.
The repository is split into three documentation handbooks so readers can keep
product behavior, operations execution, and maintainer control-plane concerns
separate from the start.

This page is the system landing page. It should answer the first high-value
questions quickly:

- what atlas is for
- where runtime authority lives
- how operations are executed and validated
- which handbook owns the next question

Use the top navigation like the other Bijux repositories:

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

<div class="bijux-callout"><strong>Start with ownership boundaries, not the file tree.</strong>
Repository docs explain atlas runtime behavior and contracts. Operations docs
explain stack, Kubernetes, observability, load, and release operations.
Maintainer docs explain control-plane automation, governance, and repository
health contracts.</div>

<div class="bijux-panel-grid">
  <div class="bijux-panel"><h3>Repository</h3><p>Runtime architecture, interfaces, contracts, ingest/query behavior, and package ownership.</p></div>
  <div class="bijux-panel"><h3>Operations</h3><p>Cluster and stack operations, install profiles, observability posture, release operations, and evidence expectations.</p></div>
  <div class="bijux-panel"><h3>Maintainer</h3><p>Governance, workflow ownership, repository diagnostics, policy enforcement, and release readiness controls.</p></div>
</div>

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="bijux-atlas/">Open Repository</a>
<a class="md-button" href="bijux-atlas-ops/">Open Operations</a>
<a class="md-button" href="bijux-atlas-dev/">Open Maintainer</a>
</div>

## System Snapshot

```mermaid
flowchart LR
    ingest[Ingest and Catalog Build] --> runtime[Atlas Runtime]
    runtime --> interfaces[CLI and HTTP Interfaces]
    runtime --> contracts[Contracts and Policy Gates]
    ops[Operations Surface] --> runtime
    ops --> observability[Observability and Load]
    ops --> delivery[Release and Deployment]
    maintain[Maintainer Control Plane] --> contracts
    maintain --> delivery
```

## Why This Landing Page Matters

Many readers stop here. This page therefore carries core system intent, not
just links:

- `bijux-atlas` is a runtime product, not only a data or docs repository
- `bijux-atlas-ops` is part of the product surface, not an appendix
- release and deployment are governed by explicit workflow and policy contracts
- maintainer controls exist to keep runtime and operations behavior explainable

## Start Here By Goal

| Goal | Open | Why |
| --- | --- | --- |
| Understand the runtime product | [Repository](bijux-atlas/index.md) | Runtime behavior, interfaces, contracts, and architecture are owned here. |
| Operate atlas in cluster or stack environments | [Operations](bijux-atlas-ops/index.md) | Install, render, validate, observe, and troubleshoot operations here. |
| Verify delivery, policy, and governance posture | [Maintainer](bijux-atlas-dev/index.md) | Repository controls, workflow ownership, and evidence rules live here. |

## Operations Depth On This Site

The operations handbook is intended for real execution decisions, not only
reference reading. It includes:

- stack and Kubernetes surfaces
- install and profile guidance
- release and distribution channels
- observability, traces, alerts, and diagnostics
- load and reproducibility contracts

If your question touches deployment risk, runtime safety, rollback, or
performance confidence, start with [Operations](bijux-atlas-ops/index.md).

## Release and Verification Lanes

Current release-critical lanes and documentation lanes for atlas are:

- `repo/ci`
- `deploy-docs`
- `release-crates`
- `release-ghcr`
- `release-github`

These lanes are represented in the badge row above and are the primary
publication and confidence signals for this repository.

## Handbook Map

- [Repository](bijux-atlas/index.md)
- [Operations](bijux-atlas-ops/index.md)
- [Maintainer](bijux-atlas-dev/index.md)

## Purpose

Use this page to understand atlas system intent quickly, pick the correct
handbook branch, and move to the source-backed pages that carry detailed proof.

## Stability

This page is part of the canonical docs spine. Keep it aligned with the
published handbook roots, active release and docs lanes, and the current atlas
runtime and operations surfaces.
