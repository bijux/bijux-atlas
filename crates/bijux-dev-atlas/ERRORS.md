# ERRORS (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

## Purpose

This page maps operator-facing error surfaces for the dev control-plane and points to the
authoritative contracts.

## Error Surfaces

- CLI usage / argument errors:
  - emitted by `cli` parsing and command dispatch
  - machine-readable shape available with `--json` where supported
- Check execution/report errors:
  - emitted by `check`, `workflows`, and `gates` command families
  - summarized through `RunReport` / `RunSummary`
- Adapter effect denials:
  - `fs_write`, `subprocess`, `git`, `network` denials when capabilities are not explicitly allowed
- Docs/configs/ops command contract errors:
  - returned as structured JSON payloads for `--format json`

## Exit Code Authority

- Canonical exit code behavior and taxonomy: `crates/bijux-dev-atlas/ERROR_TAXONOMY.md`
- Human-facing examples: `crates/bijux-dev-atlas/EXAMPLES.md`
- Command contract reference: `crates/bijux-dev-atlas/CONTRACT.md`

## Notes

- `ERRORS.md` is a quick entrypoint for operators and reviewers.
- `ERROR_TAXONOMY.md` remains the source of truth for exact classifications and mappings.
