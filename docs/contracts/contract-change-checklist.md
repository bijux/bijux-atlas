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

- [ ] `format_contracts.py` passes.
- [ ] `generate_contract_artifacts.py` run and committed.
- [ ] `check_all.sh` passes.
- [ ] `check_breaking_contract_change.py` output reviewed.
- [ ] `make docs-freeze` passes.
- [ ] `make docs` passes.

## Failure modes

Unchecked boxes mean the change is incomplete and must not merge.

## Examples

```bash
$ ./scripts/contracts/check_all.sh
$ make docs docs-freeze
```

Expected output: all checks pass.

## How to verify

```bash
$ rg -n "\[ \]" docs/contracts/contract-change-checklist.md
```

Expected output: checklist template contains required gate items for reviewers.

## See also

- [SSOT Workflow](SSOT_WORKFLOW.md)
- [Contract Diff Review](contract-diff.md)
- [Terms Glossary](../_style/TERMS_GLOSSARY.md)
