---
title: Run Your First Queries
audience: mixed
type: how-to
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Run Your First Queries

After the server is running, the quickest way to understand Atlas is to hit a small set of stable endpoints that show identity, data lookup, and query validation behavior.

## First Query Set

```mermaid
flowchart LR
    Version[Version endpoint] --> Confidence[Server identity]
    Datasets[Datasets endpoint] --> Discovery[Published datasets]
    Genes[Genes endpoint] --> Retrieval[Gene lookup]
    Validate[Query validation endpoint] --> Guardrails[Query validation]
```

This first-query set is intentionally small. It checks identity, dataset discovery, a simple data
lookup, and request validation without asking a new reader to understand the entire query surface at
once.

## 1. Check Server Identity

```bash
curl -s http://127.0.0.1:8080/v1/version
```

This confirms you are talking to a live Atlas runtime rather than only a health endpoint.

## 2. Discover Published Datasets

```bash
curl -s "http://127.0.0.1:8080/v1/datasets"
```

This confirms the server can see the catalog and published dataset identity that came from your built sample store.

## 3. Run a Simple Gene Query

```bash
curl -s "http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1"
```

## 4. Run a Gene Count Query

```bash
curl -s "http://127.0.0.1:8080/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1"
```

```mermaid
sequenceDiagram
    participant Client
    participant Router
    participant QueryLayer
    participant Store
    Client->>Router: GET /v1/genes
    Router->>QueryLayer: validate and normalize params
    QueryLayer->>Store: resolve dataset state
    Store-->>QueryLayer: matching records
    QueryLayer-->>Client: structured response
```

This sequence makes the query path easier to reason about. A request is validated, resolved against
published dataset state, executed, and then returned as structured output.

## 5. Validate a Query Without Executing It

```bash
curl -s \
  -H 'Content-Type: application/json' \
  -d '{"release":"110","species":"homo_sapiens","assembly":"GRCh38","gene_id":"g1","limit":"1"}' \
  http://127.0.0.1:8080/v1/query/validate
```

This endpoint is useful when you want to understand whether a request is well-formed before you depend on full execution behavior.

## What These Queries Teach You

```mermaid
flowchart TD
    Version[Version] --> Runtime[Runtime identity]
    Datasets[Datasets] --> Catalog[Catalog visibility]
    Genes[Genes] --> Data[Queryable dataset state]
    Validate[Query validate] --> Rules[Request guardrails]
```

This summary map helps readers interpret what each endpoint proves. The goal is not just to get
responses, but to learn which part of the system each response exercises.

- `v1/version` proves the runtime is alive
- `v1/datasets` proves the store and catalog are wired
- `v1/genes` proves the query path is working with an explicit selector
- `v1/query/validate` proves request-shape rules are active before execution

## What to Do Next

- read [Query Workflows](query-workflows.md)
- read [Runtime Surfaces](../foundations/runtime-surfaces.md)
- read [Documentation Map](../foundations/documentation-map.md)

## Common First-Query Mistakes

- querying with release, species, or assembly values that do not match the published sample dataset
- assuming `healthz` success already proves catalog and query resolution
- treating an empty result as a server failure before checking dataset identity filters

## Purpose

This page explains the Atlas material for run your first queries and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
