# Make Wrapper Delegation Audit

## Scope

This audit records `make` surface status for delegation doctrine enforcement.

## Classification (wrapper vs logic)

| Target | File | Classification | Notes |
| --- | --- | --- | --- |
| `checks-all` | `make/root.mk` | wrapper | delegates to `bijux-dev-atlas checks run` |
| `contracts-all` | `make/contracts.mk` | wrapper | delegates to `bijux-dev-atlas contract run --mode all` |
| `tests-all` | `make/root.mk` | wrapper | delegates to `bijux-dev-atlas tests run --mode all` |
| `docs-build` | `make/docs.mk` | wrapper | delegates to `bijux-dev-atlas docs build` |
| `docs-serve` | `make/docs.mk` | wrapper | delegates to `bijux-dev-atlas docs serve` |
| `ops-validate` | `make/ops.mk` | wrapper | delegates to `bijux-dev-atlas ops validate` |
| `release-plan` | `make/root.mk` | wrapper | delegates to `bijux-dev-atlas release plan` |
| `openapi-generate` | `make/root.mk` | wrapper | delegates to `bijux-dev-atlas api contract` |

## Outcome

- Named governance wrappers are single-command delegations.
- Wrapper verification command is available at `bijux-dev-atlas make wrappers verify`.
