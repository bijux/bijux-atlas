# Contract Diff Review

- Owner: `docs-governance`

## What

Defines how to review registry deltas before merge.

## Why

Contract diffs are API surface changes and require deterministic review.

## Scope

Applies to every change under `docs/contracts/*.json`.

## Non-goals

Does not replace automated breakage detection.

## Contracts

- Compare registry JSON diff first.
- Compare generated docs/code diff second.
- Validate compatibility level against registry stability.

## Failure modes

Missing or unordered review causes drift and compatibility regressions.

## Examples

```bash
$ rg --files docs/contracts | rg '\.json$'
$ bijux dev atlas contracts check --checks breakage
```

Expected output: contract files are listed and breakage checker passes for additive changes.

## How to verify

```bash
$ bijux dev atlas contracts check --checks breakage
```

Expected output: no breaking contract change unless intentionally approved.

## See also

- [SSOT Workflow](ssot-workflow.md)
- [Contract Change Checklist](contract-change-checklist.md)
- [Terms Glossary](../_style/terms-glossary.md)
