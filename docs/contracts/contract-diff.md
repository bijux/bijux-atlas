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
$ git diff -- docs/contracts/*.json
$ git diff -- docs/contracts/*.md docs/_generated/contracts crates/*/src/generated
```

Expected output: reviewers can map each SSOT change to generated outputs.

## How to verify

```bash
$ ./scripts/contracts/check_breaking_contract_change.py
```

Expected output: no breaking contract change unless intentionally approved.

## See also

- [SSOT Workflow](SSOT_WORKFLOW.md)
- [Contract Change Checklist](contract-change-checklist.md)
- [Terms Glossary](../_style/TERMS_GLOSSARY.md)
