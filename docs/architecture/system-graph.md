# Architecture Diagram

- Owner: `docs-governance`
- Stability: `evolving`

## What

Text+image view of the primary architecture flow.

## Why

Keeps one canonical diagram target for docs, reviews, and onboarding.

## Scope

Contract registries, services, storage, and ops validation loops.

## Non-goals

Does not encode every crate/module edge.

## Contracts

- Diagram source of truth: this page.
- SVG artifact: `docs/_assets/system-graph.svg`.

## Diagram (text)

`docs/contracts/*.json` -> `atlasctl contracts generate --generators artifacts` -> generated docs/code
`crates/*` -> API + ingest/runtime behavior -> k8s chart deploy
`ops/*` -> stack/deploy/smoke/load/observability checks -> `artifacts/ops/*`

![System graph](../_assets/system-graph.svg)

## Failure modes

If diagram drifts from workflows, operators may run wrong validation sequences.

## How to verify

```bash
$ make docs
```

Expected output: diagram link resolves and docs checks pass.

## See also

- [Repository Overview](repo-overview.md)
- [Contracts SSOT](../contracts/INDEX.md)
- [Operations Index](../operations/INDEX.md)
