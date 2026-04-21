---
title: bijux-atlas Documentation
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-21
---

# Bijux Atlas

`bijux-atlas` turns validated source inputs into immutable release artifacts and
serves them through stable query surfaces. It is built for teams that need
traceable data workflows, predictable runtime behavior, and operations that can
be audited and repeated.

Atlas is one product with three connected surfaces:

- **Repository**: runtime architecture, interfaces, workflows, and contracts
- **Operations**: stack, Kubernetes, observability, load, rollout, and recovery
- **Maintainer**: governance, policy controls, and repository health gates

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

<div class="bijux-panel-grid">
  <div class="bijux-panel"><h3>What Atlas Gives You</h3><p>Deterministic ingest, immutable artifacts, explicit catalog state, and stable query behavior.</p></div>
  <div class="bijux-panel"><h3>What Operations Gives You</h3><p>Clear rollout controls, incident response paths, observability contracts, and load budgets.</p></div>
  <div class="bijux-panel"><h3>What Maintainer Controls Give You</h3><p>Repeatable release gates, governance checks, and evidence-backed publication decisions.</p></div>
</div>

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="bijux-atlas/">Open Repository Handbook</a>
<a class="md-button" href="bijux-atlas-ops/">Open Operations Handbook</a>
<a class="md-button" href="bijux-atlas-dev/">Open Maintainer Handbook</a>
</div>

## Atlas Flow

```mermaid
flowchart LR
    source[Source Inputs] --> validate[Validation]
    validate --> build[Artifact Build]
    build --> publish[Catalog and Store Publish]
    publish --> serve[Runtime Serving]
    serve --> observe[Observability and Load Signals]
    observe --> release[Release Promotion]
```

## Choose Your Path

| If you need to... | Start here |
| --- | --- |
| understand runtime behavior, architecture, or API and CLI contracts | [Repository](bijux-atlas/index.md) |
| deploy, operate, debug, or recover atlas in real environments | [Operations](bijux-atlas-ops/index.md) |
| verify policy, governance, or release readiness | [Maintainer](bijux-atlas-dev/index.md) |

## Operations Pages Worth Opening First

| Operational question | Page |
| --- | --- |
| How is the system wired? | [Stack Service Topology](bijux-atlas-ops/stack/service-topology.md) |
| What protects rollout safety? | [Kubernetes Rollout Safety](bijux-atlas-ops/kubernetes/rollout-safety.md) |
| How do we run incident response? | [Observability Incident Response](bijux-atlas-ops/observability/incident-response.md) |
| What are the load pass/fail thresholds? | [Load Thresholds and Budgets](bijux-atlas-ops/load/thresholds-and-budgets.md) |
| What proves release trust? | [Release Signing and Provenance](bijux-atlas-ops/release/signing-and-provenance.md) |
| What goes into release approval evidence? | [Release Evidence](bijux-atlas-ops/release/release-evidence.md) |

## Current Release Health Signals

The main publication and confidence lanes are:

- `repo/ci`
- `deploy-docs`
- `release-crates`
- `release-ghcr`
- `release-github`

These are the signals shown in the badges above and the primary indicators of
release readiness for atlas.

## Handbook Map

- [Repository](bijux-atlas/index.md)
- [Operations](bijux-atlas-ops/index.md)
- [Maintainer](bijux-atlas-dev/index.md)

## Purpose

Use this page to quickly understand what atlas does, where operations depth
lives, and where to continue based on your current goal.

## Stability

This page is part of the canonical docs spine. Keep it aligned with active
runtime surfaces, operational workflows, and release lanes.
