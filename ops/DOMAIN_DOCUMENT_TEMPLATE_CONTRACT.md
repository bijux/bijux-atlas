# Ops Domain Document Template Contract

- Owner: `bijux-atlas-operations`
- Purpose: Canonical required structure for authored `ops/<domain>/CONTRACT.md` and `ops/<domain>/README.md` files.
- Consumers: `checks_ops_domain_contract_structure`
- Authority Tier: `machine`
- Audience: `contributors`

## Scope

This contract defines the minimum required authored structure for domain-level ops documents.
It is the canonical template contract for enforcement, not a generated output.

## Domain CONTRACT.md Required Metadata

- `- Owner:`
- `- Purpose:`
- `- Consumers:`
- `- contract_version: \`...\``
- `- contract_taxonomy: \`structural|behavioral|hybrid\``

## Domain CONTRACT.md Required Sections

- `## Contract Taxonomy`
- `## Authored vs Generated`
- `## Invariants`
- `## Enforcement Links`
- `## Runtime Evidence Mapping`

## Domain README.md Required Metadata

- `- Owner:`
- `- Purpose:`
- `- Consumers:`

## generated/README.md Required Metadata

- `- Producer:`
- `- Regenerate:`

## Enforcement

- `checks_ops_domain_contract_structure`
