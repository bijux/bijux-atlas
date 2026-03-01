# Ops Error Catalog

Repo-law violations are executable contract failures, not ad hoc review comments.

- Canonical gate: `bijux dev atlas contracts root --mode static`
- Scope: repo root governance, required checks, tracked-artifact policy, and canonical entrypoints

## Repo Law Errors

| Code | Meaning | Primary contract surface | Remediation |
| --- | --- | --- | --- |
| `REPO-LAW-001` | A root entry exists outside `ops/inventory/root-surface.json`. | `ROOT-001`, `ROOT-016` | Remove the entry or update the governed root manifest and related contracts in the same change. |
| `REPO-LAW-002` | A legacy script surface or unmanaged root control path was introduced. | `ROOT-003`, `ROOT-006`, `ROOT-034` | Route the behavior through `bijux dev atlas` and keep root wrappers thin. |
| `REPO-LAW-003` | Tracked files or uncontrolled outputs appeared under `artifacts/`. | `ROOT-014` | Move tracked examples to `ops/_generated.example/` and keep runtime evidence under `artifacts/run/<run_id>/`. |
| `REPO-LAW-004` | Required branch checks or repo-law governance docs drifted from executable policy. | `ROOT-045`, `ROOT-046` | Update `.github/required-status-checks.md` and governance docs together with the enforcing contracts. |
