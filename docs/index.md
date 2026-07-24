---
title: bijux-atlas Documentation
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Bijux Atlas

Bijux Atlas turns governed GFF3 and FASTA inputs into immutable genomic dataset
releases and serves them through Rust libraries, command workflows, and a
versioned HTTP API. Its documentation follows the decisions needed to build,
publish, operate, and trust those releases.

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

## Choose by Outcome

| Outcome | Handbook |
| --- | --- |
| understand dataset construction, publication, queries, and interfaces | [Atlas product](bijux-atlas/index.md) |
| run the complete committed sample journey | [Run Atlas locally](bijux-atlas/workflows/run-atlas-locally.md) |
| deploy, secure, observe, load-test, or recover a target | [Atlas operations](bijux-atlas-ops/index.md) |
| interpret release evidence or distribution trust | [Release operations](bijux-atlas-ops/release/index.md) |
| change repository automation, policy, or delivery | [Maintainer control plane](bijux-atlas-dev/index.md) |

## Understand the System

Atlas has two independent lifecycles that meet at a serving decision.

```mermaid
flowchart TB
    subgraph Dataset["Dataset lifecycle"]
        Source["governed source"] --> Build["build + verify"]
        Build --> Publish["publish immutable artifacts"]
        Publish --> Catalog["promote catalog identity"]
    end
    subgraph Service["Service lifecycle"]
        Software["software release"] --> Render["profile + render + admission"]
        Render --> Run["running server"]
        Run --> Qualify["security + telemetry + load + recovery"]
    end
    Catalog --> Identity["resolved serving identity"]
    Run --> Identity
    Identity --> Result["CLI or HTTP result"]
    Qualify --> Decision["promotion or recovery decision"]
```

A published dataset may have no qualified deployment. A healthy deployment may
observe the wrong catalog generation. A trustworthy result therefore binds:

- the dataset tuple `release/species/assembly`;
- manifest and artifact hashes;
- software, configuration, chart, profile, and target identity; and
- the request, observation window, or scenario that produced the evidence.

## Read by Ownership

The documentation is divided by who makes the decision:

| Surface | Owns | Does not own |
| --- | --- | --- |
| [product](bijux-atlas/index.md) | data model, ingest, publication, query, runtime, and interfaces | deployment qualification |
| [operations](bijux-atlas-ops/index.md) | topology, Kubernetes, security, signals, load, recovery, and release trust | product semantics |
| [maintenance](bijux-atlas-dev/index.md) | repository validation, generation, governance, and delivery automation | user-facing runtime behavior |

Evidence crosses these surfaces without transferring ownership. A maintainer
command can validate a chart contract but cannot declare a target healthy. An
operator can observe a correct query but cannot redefine the dataset identity
or wire contract.

## Match Evidence to the Claim

| Evidence | Establishes | Does not establish |
| --- | --- | --- |
| schema, policy, or registry | declared structure and required fields | that behavior ran |
| checked-in fixture | validator and serialization shape | target fitness |
| execution report | observed behavior for named inputs and a run | release binding unless identities are included |
| checksums and provenance | artifact membership and lineage | operational fitness |
| consumer verification receipt | coherence and authorization of exact received bytes | upstream biological correctness |

When evidence is missing, narrow the claim rather than filling the gap with a
later green check. Readiness does not prove capacity; a load result does not
prove recovery; an internally coherent release packet does not provide an
independent trust anchor.

## Current Qualification Boundaries

The documentation records limitations that affect present claims:

- production-oriented overlays are not all represented in executable lifecycle
  scenarios;
- registered rollout-under-load suites do not currently have executable control
  runners;
- administrative endpoint classification does not cover every registered route;
- the repository defines recovery contracts but does not provide an operational
  production backup schedule or restore runner; and
- the GHCR release path publishes compressed bundles as OCI artifacts, not
  evidence of runnable container images.

These are decision boundaries, not documentation footnotes. Follow the linked
operations pages before describing a deployment or release as qualified.

## Continue with a Concrete Question

- [What Atlas is](bijux-atlas/foundations/what-atlas-is.md)
- [Dataset and query workflows](bijux-atlas/workflows/index.md)
- [CLI, HTTP, OpenAPI, and configuration](bijux-atlas/interfaces/index.md)
- [Runtime and storage architecture](bijux-atlas/runtime/index.md)
- [Deployment and service topology](bijux-atlas-ops/stack/index.md)
- [Kubernetes delivery](bijux-atlas-ops/kubernetes/index.md)
- [Security assurance](bijux-atlas-ops/security/index.md)
- [Observability](bijux-atlas-ops/observability/index.md)
- [Load and resilience](bijux-atlas-ops/load/index.md)
- [Release and recovery](bijux-atlas-ops/release/index.md)
