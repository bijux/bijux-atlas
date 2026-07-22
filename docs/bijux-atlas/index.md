---
title: Atlas Product Overview
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# bijux-atlas

Atlas delivers immutable genomic dataset releases and stable query surfaces
over published catalog state. The product boundary runs from source admission
through artifact construction, publication, serving, and compatibility.

```mermaid
flowchart LR
    Inputs[Governed GFF3 and FASTA] --> Ingest[Validate and normalize]
    Ingest --> Build[Build immutable artifacts]
    Build --> Verify[Verify artifact identity and integrity]
    Verify --> Store[Publish immutable store payload]
    Store --> Catalog[Promote catalog identity]
    Catalog --> Query[Execute gene, transcript, sequence, and diff queries]
    Query --> Interfaces[CLI, HTTP, OpenAPI, and Rust APIs]
```

## Product Capabilities

Atlas is the repository-owned product surface for:

- ingesting governed GFF3 and FASTA inputs into immutable dataset artifacts;
- publishing those artifacts into a serving store and promoting their catalog identity;
- serving dataset identity, gene, transcript, sequence, and diff workflows;
- exposing a stable CLI, HTTP, and OpenAPI surface around those artifacts.

The Atlas product surface is carried by a split crate set.
`bijux-atlas-runtime` owns orchestration. `bijux-atlas` preserves the historical
import path. The CLI, server, and API crates own the direct binaries. Leaf
crates own ingest, query, model, core, store, and operations contracts.

| Capability | Product boundary |
| --- | --- |
| dataset construction | validates and normalizes supported GFF3 and FASTA into release artifacts. |
| publication | moves complete, verified artifacts into an immutable store payload. |
| promotion | exposes published release, species, and assembly identity through the catalog. |
| discovery | resolves catalog, dataset, and endpoint identity without redefining release truth. |
| query | serves genes, counts, transcripts, sequence regions, and release comparisons. |
| delivery | exposes direct binaries, split Rust crates, HTTP routes, and generated OpenAPI. |
| compatibility | versions wire shapes, output, configuration, plugins, artifacts, and crates. |

## Crate Architecture

```mermaid
flowchart TB
    Core[core and model] --> Ingest[ingest]
    Core --> Query[query]
    Core --> Store[store]
    Ingest --> Runtime[runtime orchestration]
    Query --> Runtime
    Store --> Runtime
    Runtime --> CLI[CLI binary]
    Runtime --> Server[server binary]
    API[API contracts and OpenAPI] --> Server
    Runtime --> Compat[compatibility alias crate]
```

Ownership stays split so consumers can depend on the narrowest durable surface:

- dataset identity, gene, transcript, and diff meaning live primarily under
  `crates/bijux-atlas-model/src/`
- ingest-time normalization and artifact construction live under
  `crates/bijux-atlas-ingest/src/engine/`
- query semantics live under `crates/bijux-atlas-query/src/engine/`
- runtime assembly, store ports, policy, and configuration live under
  `crates/bijux-atlas-runtime/src/app/`,
  `crates/bijux-atlas-runtime/src/domain/`, and
  `crates/bijux-atlas-runtime/src/runtime/`
- HTTP and API surface lives under
  `crates/bijux-atlas-server/src/adapters/inbound/http/`
- CLI surface and user-facing command handling live under
  `crates/bijux-atlas-cli/src/bin/`,
  `crates/bijux-atlas-server/src/bin/`, and
  `crates/bijux-atlas-api/src/bin/`
- generated API and runtime references live under `configs/generated/openapi/`
  and `configs/generated/runtime/`
- workflow examples and machine-checked contract shapes live under
  `configs/examples/` and `configs/schemas/contracts/`

## Follow a Result

```mermaid
flowchart TD
    Result[Observed query result] --> Release[Resolved release identity]
    Release --> Catalog[Catalog entry]
    Catalog --> Manifest[Artifact manifest and hashes]
    Manifest --> Inputs[Governed source identities]
    Result --> Surface[CLI or HTTP contract]
    Result --> Runtime[Runtime configuration and policy]
```

A reviewable result has two traceable paths. Its release identity leads through
the catalog and artifact manifest to governed inputs. Its shape leads to the
owning interface contract. Those paths are more useful than a generic statement
that the runtime or dataset is current.

## Locate the Broken Boundary

Atlas failures are easier to diagnose when the observed symptom is traced to
the first authority that could have produced it:

| Observation | Inspect first | Do not conclude yet |
| --- | --- | --- |
| ingest rejects a record | normalization finding and source location | all source files are invalid. |
| deep verification fails | manifest, lock, payload hash, referenced artifact | publication caused corruption. |
| published tuple is absent | catalog entry and promotion record | the payload was never published. |
| readiness is false | catalog refresh, dataset state, lifecycle mode | the process is not live. |
| query returns no rows | tuple, selector, ordering, page boundary | the dataset is missing. |
| HTTP and CLI disagree | shared result before presentation adapters | the wire contract alone is wrong. |

Start at the earliest failed boundary and preserve its identity. Skipping
directly to a later layer often turns an explicit data, publication, or catalog
problem into an ambiguous runtime symptom.

## Choose a Product Surface

Choose a path based on the question in front of you:

- start in [Foundations](foundations/index.md) for the product model, terminology, or repository scope;
- move to [Workflows](workflows/index.md) to install Atlas, build data, start a server, or run queries;
- use [Interfaces](interfaces/index.md) for commands, endpoints, flags, outputs, or environment variables;
- use [Runtime](runtime/index.md) for architecture, lifecycle, storage, request flow, or source layout;
- use [Contracts](contracts/index.md) for compatibility promises and review rules.

## Publication Boundary

Atlas is artifact-first. The runtime is not meant to serve mutable, partially
built local state directly from ad hoc ingest output. The normal path preserves
separate authorities:

```mermaid
flowchart LR
    Candidate[Built candidate] --> Verify[Validate and deeply verify]
    Verify --> Store[Publish immutable payload]
    Store --> Promote[Promote catalog entry]
    Promote --> Refresh[Runtime refreshes catalog]
    Refresh --> Result[Return result with resolved dataset identity]
```

Store publication proves that immutable bytes exist under the selected backend
contract. Catalog promotion proves discoverability. Runtime refresh proves one
instance observed that catalog state. None substitutes for another.

Serving from an ingest build directory bypasses catalog promotion, store
identity, and release provenance. A completed build is therefore necessary but
not sufficient for a serveable release.

## Contract Authorities

Stable claims are backed by four kinds of authority:

- implementation code under the owning split crates in `crates/`
- generated references under `configs/generated/`
- machine-checked contract schemas under `configs/schemas/contracts/`
- example or workflow material under `configs/examples/`

Those authorities have different force. Implementation and schemas define
behavior. Generated references expose the resolved surface. Examples teach a
supported path but do not expand the contract. A release-specific claim also
needs evidence from the owning workflow.

| Reader question | Product authority | Release-specific proof |
| --- | --- | --- |
| Which dataset identity is served? | model, artifact, store, and catalog | published manifest and store/catalog record. |
| Which queries are stable? | query, structured-output schemas, OpenAPI | released-binary contract results. |
| Which command owns an operation? | CLI tree and generated reference | released-command help or contract output. |
| Is an ingest directory serveable? | publication and artifact contracts | completed publish record, not build output. |
| Is a wire change compatible? | API and compatibility policy | report for the affected release pair. |

## Continue by Concern

- [Foundations](foundations/index.md)
- [Workflows](workflows/index.md)
- [Interfaces](interfaces/index.md)
- [Runtime](runtime/index.md)
- [Contracts](contracts/index.md)

For deployment, rollout, security, observability, load, and release decisions,
continue to [Atlas Operations](../bijux-atlas-ops/index.md). For repository
automation and contribution workflows, continue to the
[Maintainer Control Plane](../bijux-atlas-dev/index.md).
