# Ops Documentation Contract

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/inventory/authority-index.json`, `ops/inventory/**`, `ops/schema/**`, `ops/CONTRACT.md`

## Authority Tiers

- `tier0-machine`: machine-readable source of truth (`ops/inventory/**`, `ops/schema/**`, and normative structured files under `ops/**` such as json/yaml/toml).
- `tier1-normative`: minimal human-readable normative contracts in `ops/` that define rules and must be backed by checks/schemas.
- `tier2`: narrative operator/developer documentation under `docs/operations/**` that explains how/why and links to authoritative sources.

## Tier Rules

- Tier-2 pages must not restate Tier-0 or Tier-1 facts as standalone source of truth. Link to the authoritative source instead.
- Tier-2 pages must not define directory maps, command surfaces, pins, tool lists, or schema contracts except as generated reference pages.
- Tier-1 docs must stay minimal and normative; tutorials and workflow walkthroughs belong in `docs/operations/**`.
- Rules that block CI must exist in machine form (inventory/schema/check code), not prose only.

## Header Requirements

Every page under `docs/operations/**` must declare:

- `Owner`
- `Tier`
- `Audience`
- `Source-of-truth`

## Authority Exceptions

Temporary exceptions are declared only in `ops/inventory/authority-tier-exceptions.json` and must include:

- `path`
- `rule`
- `reason`
- `expires_on` (YYYY-MM-DD)

## Canonical Locations

- Narrative entrypoint: `docs/operations/index.md`
- Ops narrative glossary: `docs/_style/terms-glossary.md`
- Normative ops root contracts: `ops/*.md` (minimal Tier-1 set)

## Enforcement

- `checks_ops_docs_governance`

Related contracts: OPS-ROOT-023, OPS-ROOT-017.
