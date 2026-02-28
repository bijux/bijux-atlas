# Architecture Map

Owner: `architecture`  
Type: `concept`  
Reason to exist: provide one canonical system map for components, boundaries, and runtime direction.

## System Flow

`contracts -> crates -> runtime services -> operations evidence`

## Runtime Direction

`bijux-atlas-server -> bijux-atlas-query -> bijux-atlas-store -> immutable artifacts`

## Crate Roles

| Crate | Role | Internal Dependencies |
| --- | --- | --- |
| `bijux-atlas-api` | API surface | `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query` |
| `bijux-atlas-cli` | control and operations entrypoints | `bijux-atlas-core`, `bijux-atlas-ingest`, `bijux-atlas-model`, `bijux-atlas-policies`, `bijux-atlas-query`, `bijux-atlas-store` |
| `bijux-atlas-core` | shared core primitives | `(none)` |
| `bijux-atlas-ingest` | ingest pipeline | `bijux-atlas-core`, `bijux-atlas-model` |
| `bijux-atlas-model` | shared data model | `(none)` |
| `bijux-atlas-policies` | policy contracts | `(none)` |
| `bijux-atlas-query` | query engine | `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-policies`, `bijux-atlas-store` |
| `bijux-atlas-server` | runtime server | `bijux-atlas-api`, `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query`, `bijux-atlas-store` |
| `bijux-atlas-store` | artifact store | `bijux-atlas-core`, `bijux-atlas-model` |

## Diagram

![System graph](../_assets/system-graph.svg)

## Operational Relevance

This map defines where production changes must be implemented and which layer owns incident diagnostics.
