# Naming and Wording Conventions

- Owner: `docs-governance`

## Purpose

Define neutral, deterministic language for docs, checks, and CLI surfaces.

## Required Wording

- Use neutral terms such as `contract`, `invariant`, `SSOT`, `guarantee`, and `policy`.
- Prefer exact names of checks, commands, and files over subjective adjectives.

## Forbidden Adjectives

Legacy marketing adjectives are forbidden in repository content unless explicitly approved for historical quotes.
The canonical blocked-term list lives in policy config so docs do not duplicate or drift from policy data.

Policy config:

- `configs/policy/forbidden-adjectives.json`
- `configs/policy/forbidden-adjectives-approvals.json`

## Verification

- Run `bijux dev atlas policies forbidden-adjectives --report json`.
- Report artifact: `artifacts/reports/dev-atlas/forbidden-adjectives.json`.
