---
title: Docs Governance Entry
audience: contributor
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-01
tags:
  - governance
  - navigation
related:
  - docs/contract.md
  - docs/site-map.md
---

# Docs Governance Entry

Use this entrypoint when you are working on the documentation system itself rather than reading the product docs. It is intentionally excluded from the published MkDocs reader build.

## Canonical records

- Reader entrypoint: [Home](index.md)
- Reader map: [Docs site map](site-map.md)
- Contract surface: [Docs Contract](CONTRACT.md)
- Machine registry: `docs/registry.json`
- Section manifest: `docs/sections.json`
- Owner registry: `docs/owners.json`

## Generation policy

- Reader-facing markdown under `docs/` is tracked.
- Generated docs evidence lives under `docs/_internal/generated/`.
- Contributor audit material lives under `docs/_internal/governance/`.
- Runtime command output belongs under `artifacts/run/<run_id>/`, never under tracked reader pages.

## Control-plane commands

- Validate the docs surface: `bijux dev atlas contracts docs --mode static`
- Run the broader docs checks: `bijux dev atlas docs check --allow-subprocess`
- Refresh registry-driven generated files: `bijux dev atlas docs registry build --allow-write`

## Next steps

- Reader navigation: [Home](index.md)
- Contributor workflow: [Development](development/index.md)
- Contract reference: [Docs Contract](CONTRACT.md)
