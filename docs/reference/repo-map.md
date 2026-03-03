# Repository Map

- Owner: `bijux-atlas-docs`
- Review cadence: `quarterly`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@ff8cd5f299e568c93feec8b4d40347bf1c5a93c4`
- Source-of-truth: `docs/_internal/generated/docs-inventory.md`
- Reason to exist: point readers to the canonical repository layout guide and the governed generated inventory.

## Purpose

This page is the stable public entrypoint for repository layout questions.
It does not duplicate the generated inventory because that surface changes frequently and is maintained for
contributors, not readers.

## Entrypoints

- Read [Repository Layout](../development/repo-layout.md) for the curated explanation of where code and
  operational surfaces live.
- Use `docs/_internal/generated/docs-inventory.md` as the detailed generated inventory when you need the
  current committed filesystem snapshot.

## Verification

The docs contracts verify that the `Source-of-truth` path above exists and that the curated entrypoint keeps
pointing at the canonical repository layout page.
