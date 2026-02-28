# Generated Documentation Content Policy

Owner: `docs-governance`  
Type: `policy`  
Reason to exist: keep generated documentation surfaces immutable and machine-derived.

## Scope

- `docs/_generated/**`

## Rules

- Generated files are outputs, not authored documentation.
- Manual edits under `docs/_generated/` are not allowed.
- Generated files can be replaced only by generator commands committed in the same change set.
- Canonical narrative pages must link to generated outputs instead of copying their content.
