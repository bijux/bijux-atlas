# Naming Standard

- Owner: `docs-governance`

## What

Defines filename and title conventions for repository documentation.

## Why

Stable naming prevents navigation drift and duplicate concepts.

## Contracts

- Documentation files must use `kebab-case.md`.
- Non-kebab exceptions are `INDEX.md`, `CONCEPT_REGISTRY.md`, `DEPTH_POLICY.md`, and `DEPTH_RUBRIC.md`.
- `INDEX.md` is forbidden inside `docs/`; it is allowed only at repository/crate roots.
- Forbidden filename patterns: random acronyms, `notes`, `misc`, `HUMAN_MACHINE`.
- Forbidden temporal/task tokens in names: `phase`, `task`, `stage`, `round`, `iteration`, `tmp`, `stub`, `vnext`.
- Durable naming rubric: file names must describe what the asset is, not when it was added.
- Public shell entrypoints must use `kebab-case` (`verb-noun` style).
- Rust module files remain `snake_case`.
- Generated docs under `docs/_generated/` may use generator-defined names, but must still be linked from an `INDEX.md`.

## Durable Naming Rubric

1. Prefer domain nouns over process words.
2. Encode stable concept, not release timing.
3. Keep one concept per filename.

## Rename Patterns

- Shell entrypoints: `phase-2-cleanup.sh` -> `cache-cleanup.sh`
- Docs: `task-list.md` -> `api-compatibility-checklist.md`
- Tests: `round3-overload.rs` -> `overload-shedding.rs`
- Fixtures: `tmp-fixture.gff3` -> `edgecase-missing-parent.gff3`
- k6 suites: `stage-load.js` -> `warm-steady-load.js`

## See also

- [Structure Templates](structure-templates.md)
- [Writing Rules](writing-rules.md)
- [Depth Policy](DEPTH_POLICY.md)
