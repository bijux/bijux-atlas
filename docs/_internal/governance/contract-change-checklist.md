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
- [ ] `make contracts` passes.
- [ ] `check_breaking_contract_change.py` output reviewed.
- [ ] Any new relaxation is explicitly registered in `configs/policy/policy-relaxations.json` with owner/justification/expiry and `ATLAS-EXC-*` reference tag in code.
- [ ] Any allowlist relaxation is scoped to `dataset_identity` + `artifact_hash` (no repo-wide or name-only allowlists).
- [ ] `make contracts-docs` passes.
- [ ] `make docs` passes.

## Failure modes

Unchecked boxes mean the change is incomplete and must not merge.

## Examples

```bash
$ make contracts
$ make docs docs-freeze
```

Expected output: all checks pass.

## How to verify

```bash
$ rg -n "\[ \]" docs/_internal/governance/contract-change-checklist.md
```

Expected output: checklist template contains required gate items for reviewers.

## See also

- [SSOT Workflow](contract-ssot-workflow.md)
- [Contract Diff Review](../../reference/contracts/contract-diff.md)
- [Glossary](../../glossary.md)
