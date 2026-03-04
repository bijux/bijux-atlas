---
title: Reference
audience: user
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-01
tags:
  - reference
  - facts
related:
  - docs/api/index.md
  - docs/operations/index.md
source:
  - docs/_internal/registry/registry.json
  - configs/schema-map.json
---

# Reference

- Owner: `docs-governance`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: provide the stable factual portal for Atlas commands, configs, schemas, and contracts.

## Why you are reading this

Facts live here. Procedures do not. Use this section for stable names, keys, schemas, contracts, and inventories that other guides link to.

## Categories

- Commands and automation: [Commands](commands.md), [Command inventory](command-inventory.md)
- Distribution and package surfaces: [Crates reference](crates.md), [Docker reference](docker.md), [Ops artifacts reference](ops.md)
- Runtime definitions: [Configs reference](configs.md), [Runtime config](runtime/config.md), [Errors reference](errors.md), [Schemas reference](schemas.md)
- Config governance facts: [Schema versioning policy](schema-versioning-policy.md), [Config keys reference](config-keys-reference.md)
- Contracts and registries: [Contracts](contracts/index.md), [Reports](reports/index.md), [Dataset operations](dataset-operations.md), [Ingest reproducibility](ingest/reproducibility.md)
- Governance facts: [Governance reference](governance.md)
- Release planning: [Release planning reference](release-plan.md)
- Crate release policy: [Crate release policy](crate-release-policy.md)
- MSRV policy: [MSRV policy](msrv-policy.md)
- Crate feature flags: [Crate feature flags](crate-feature-flags.md)
- Crate dependency graph: [Crate dependency graph](crate-dependency-graph.md)
- Breaking change checklist: [Breaking change checklist](breaking-change-checklist.md)
- Query semantics: [Querying reference](querying/index.md), [Pagination](querying/pagination.md)
- Dataset semantics: [Dataset and artifact reference](datasets/index.md)
- Working examples: [Examples index](examples/index.md)

## Entry Points

- Start here for stable definitions, commands, and contract links.
- Leave this section when you need a procedure instead of a definition.

## Reading rule

- If you need a procedure, use the API or Operations sections instead of this reference portal.
- If you need a stable definition, link here instead of duplicating prose.
- Machine-readable JSON stays in contract registries and generated artifacts, not in narrative guides.

## Next steps

- [Glossary](../glossary.md)
