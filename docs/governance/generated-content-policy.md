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
- `docs/index.md` and section entrypoints must never use generated files as navigation targets.

## File Header Warning

Generated markdown outputs must start with a warning that they are output-only artifacts and must not be edited manually.
