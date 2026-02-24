# Contract Change Checklist

- Owner: `docs-governance`

## What

Checklist mapping contract edits to CI gates.

## Why

Makes contract updates executable instead of manual discipline.

## Scope

Applies to all contract registry and generated output changes.

## Non-goals

Does not supersede required reviewer approval policy.

## Contracts

- [ ] `bijux dev atlas check run --suite ci_fast` passes.
- [ ] `bijux dev atlas docs build` run and committed.
- [ ] `make ssot-check` passes.
- [ ] `check_breaking_contract_change.py` output reviewed.
- [ ] Any new relaxation is explicitly registered in `configs/policy/policy-relaxations.json` with owner/justification/expiry and `ATLAS-EXC-*` reference tag in code.
- [ ] Any allowlist relaxation is scoped to `dataset_identity` + `artifact_hash` (no repo-wide or name-only allowlists).
- [ ] `make docs-freeze` passes.
- [ ] `make docs` passes.

## Failure modes

Unchecked boxes mean the change is incomplete and must not merge.

## Examples

```bash
$ make ssot-check
$ make docs docs-freeze
```

Expected output: all checks pass.

## How to verify

```bash
$ rg -n "\[ \]" docs/contracts/contract-change-checklist.md
```

Expected output: checklist template contains required gate items for reviewers.

## See also

- [SSOT Workflow](ssot-workflow.md)
- [Contract Diff Review](contract-diff.md)
- [Terms Glossary](../_style/terms-glossary.md)
