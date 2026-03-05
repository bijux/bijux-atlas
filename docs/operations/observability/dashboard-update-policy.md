# Dashboard Update Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`

## Policy

- Dashboard edits require schema validation and rendered diff review.
- New panels must include a diagnostic purpose.
- Removed panels require replacement evidence in the same category.
- Dashboard releases must include installation and rollback notes.
- Dashboard JSON changes must be traceable in commit history.
