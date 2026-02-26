# Operations Performance Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## Topics

- Ops budgets SSOT: `budgets.md`

## What

Canonical entrypoint for performance-oriented operations policy docs.

## Why

Keeps budgets and performance guardrails discoverable from one stable location.

## Scope

Runtime and workflow performance budgets used by operations gates.

## Non-goals

Does not define endpoint-level API semantics or crate internals.

## Contracts

- [Ops Budgets](budgets.md)

## Failure modes

Missing budget documentation causes unclear lane failures and drift in performance guardrails.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass with no orphan docs.

## See also

- [Operations Index](../INDEX.md)
