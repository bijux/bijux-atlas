---
title: Governance Documentation Structure
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - docs
---

# Governance documentation structure

## Top-level layout

- `docs/governance/`: public governance policy and process
- `docs/_internal/governance/`: internal execution and maintenance procedures
- `configs/governance/`: machine-validated governance registries and schemas
- `ops/governance/`: operational fixtures and evidence data

## Authoring rule

Policy statements must exist in `docs/governance/` before enforcement rules are added to `configs/governance/enforcement/rules.json`.
