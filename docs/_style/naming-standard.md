# Naming Standard

- Owner: `docs-governance`

## What

Defines filename and title conventions for repository documentation.

## Why

Stable naming prevents navigation drift and duplicate concepts.

## Contracts

- Documentation files must use `kebab-case.md`.
- The only non-kebab exception is `INDEX.md` (section entrypoint files).
- `INDEX.md` is forbidden inside `docs/`; it is allowed only at repository/crate roots.
- Forbidden filename patterns: random acronyms, `notes`, `misc`, `HUMAN_MACHINE`.
- Generated docs under `docs/_generated/` may use generator-defined names, but must still be linked from an `INDEX.md`.

## See also

- [Structure Templates](structure-templates.md)
- [Writing Rules](writing-rules.md)
- [Depth Policy](DEPTH_POLICY.md)
