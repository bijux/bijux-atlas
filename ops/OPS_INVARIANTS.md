# Ops Invariants

- Owner: `bijux-atlas-operations`
- Purpose: list non-negotiable ops governance invariants.
- Consumers: `checks_ops_final_polish_contracts`

## Core Invariants

- Ops is specification-only.
- Generated artifacts must include schema_version and generated_by.

## Decision Rules

- Contract changes require matching inventory and schema updates.

## Enforcement Links

- checks_ops_final_polish_contracts
