# Suites And Registries

- Owner: `team:atlas-governance`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`

## Purpose

Suites define execution order and operator-facing entrypoints.
Registries define the durable metadata for checks and contracts.

## Components

- `configs/governance/suites/*.suite.json`: suite membership and execution metadata.
- `configs/governance/checks.registry.json`: checks source of truth.
- `configs/governance/contracts.registry.json`: contracts source of truth.
- `configs/governance/check-groups.json`: terminal and scheduler grouping for checks.
- `configs/governance/contract-groups.json`: terminal and scheduler grouping for contracts.
- `configs/governance/tags.json`: allowed tag taxonomy.

## Validation Surfaces

- `bijux dev atlas governance validate`
- `bijux dev atlas registry status`
- `bijux dev atlas registry doctor`
- `bijux dev atlas suites list`
- `bijux dev atlas suites describe --suite checks`

## Generated Artifacts

- `artifacts/governance/registry-status.json`
- `artifacts/governance/registry-status.md`
- `artifacts/governance/registry-work-remaining.json`
- `artifacts/governance/registry-missing-fields.json`

## Rules

- A suite entry must resolve to a real registry id.
- A registry entry must keep one durable owner.
- Checks and contracts must keep stable ids and stable report paths.
- Registry changes must keep Make and control-plane commands resolvable.
