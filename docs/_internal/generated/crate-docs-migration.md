# Crate docs migration record

- Owner: `docs-governance`
- Type: `internal`
- Stability: `frozen`
- Last reviewed: `2026-03-01`

This one-time record captures the crate docs normalization migration.

## Applied normalization

- Crate root markdown policy tightened to `README.md` and `CONTRACT.md` only.
- Legacy crate-root docs were moved into `crates/<crate>/docs/`.
- Crate docs filenames were normalized to lowercase kebab-case.
- Broken internal crate-doc references were updated to normalized paths.

## Contract coverage

- Contracts enforced by `bijux-dev-atlas` crates domain:
  - root markdown allowlist
  - docs file budget
  - kebab-case filenames
  - relative link integrity
  - README required sections
  - CONTRACT required sections

## Follow-up debt

- Bring all crate docs directories under the max-15-file budget.
