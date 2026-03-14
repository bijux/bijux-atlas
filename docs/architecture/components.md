# Components

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define component responsibilities with clear runtime and control-plane boundaries.

## Runtime Components

- `bijux-atlas-ingest`: validates source inputs and writes deterministic artifacts.
- `bijux-atlas-store`: manages serving-store persistence and artifact access.
- `bijux-atlas-query`: provides deterministic query planning and execution.
- `bijux-atlas-api`: exposes transport-level contracts for query behavior.
- `bijux-atlas-server`: hosts API runtime, caching, and readiness behavior.
- `bijux-atlas`: provides runtime-facing command workflows for operators and users.
- `bijux-atlas-client`: provides Rust client SDK integration surface for runtime consumers.

## Control-Plane Components

- `bijux-dev-atlas`: runs checks, docs validation, policies, and ops lanes.
- `ops/*` inventory: defines operator entrypoints and surface contracts.
- governance docs/contracts: enforce non-runtime policy and review controls.
- `bijux-atlas-bench`: runs governed benchmark scenarios and performance evidence generation.

## Shared Foundation Components

- `bijux-atlas-core`: cross-cutting primitives and invariants.
- `bijux-atlas-model`: shared domain structures and schema-level contracts.
- `bijux-atlas-policies`: policy tables and evaluation rules.

## What Never Happens

- Runtime crates do not bypass control-plane contracts for release-critical actions.
- API and query layers do not mutate immutable release artifacts.
- Control-plane lanes do not hide effectful steps behind undocumented scripts.

## Limits and non-goals

- Component boundaries are not designed for cross-layer hotfix shortcuts.
- Components do not guarantee zero coordination cost for bypassed architecture rules.

## Operational Relevance

Clear component boundaries reduce incident triage time and prevent cross-layer hotfixes.

## What This Page Is Not

This page is not a crate-by-crate API reference.

## What to Read Next

- [Architecture](index.md)
- [Boundaries](boundaries.md)
- [Dataflow](dataflow.md)
- [Crates Map](crates-map.md)
- [Glossary](../glossary.md)

## Terminology used here

- Artifact: [Glossary](../glossary.md)
- Control-plane: [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
