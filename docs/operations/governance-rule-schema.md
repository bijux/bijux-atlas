---
title: Governance Rule Schema
audience: operator
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - schema
---

# Governance rule schema

Schema source: `configs/governance/enforcement/rules.schema.json`

## Required rule fields

- `id`
- `title`
- `severity`
- `classification`
- `rule_type`

## Enumerations

- `severity`: `low`, `medium`, `high`
- `classification`: `repository`, `documentation`, `registry`, `deployment`
- `rule_type`: includes required/prohibited file checks, registry completeness checks, deployment artifact checks, and docs navigation checks.
