# atlasctl Deletion Plan

Cutoff date: `2026-03-31`

## Scope

Remove the legacy Python control-plane package (`packages/atlasctl`) and all repository call-sites, then lock reintroduction behind repo checks.

## Exit Criteria

- `packages/atlasctl/` is deleted.
- No `./bin/atlasctl` shim exists.
- No `atlasctl` references remain in:
  - `.github/workflows/`
  - `makefiles/`
  - `docs/` (except historical notes explicitly marked)
  - `configs/`
- No `artifacts/reports/atlasctl/` paths are created by CI or local workflows.
- CI lanes use `cargo run -q -p bijux-dev-atlas -- ...` or `make` wrappers that delegate to `DEV_ATLAS`.
- After the cutoff date, no new `atlasctl` references may be introduced outside explicit tombstones under `docs/tombstones/atlasctl/`.

## Ordered Work

1. Cut over Makefile wrappers to `DEV_ATLAS`.
2. Cut over CI/workflow governance lanes to `make` or `bijux-dev-atlas`.
3. Add hard checks for package/shim/artifact absence.
4. Delete `packages/atlasctl/`.
5. Remove atlasctl historical docs or convert to explicit archival notes.
6. Lock reintroduction with repo checks and tests.
7. Use `atlasctl-deletion-pr-checklist.md` for final deletion PR verification.

## Historical Note Policy

If a document must retain the term `atlasctl` temporarily, mark it as historical and exclude it from public operator guidance. The default policy is deletion, not permanent deprecation docs.

## Cutoff Rules (Locked)

- No new source, workflow, makefile, config, or public docs references to `atlasctl`.
- Temporary historical references are allowed only under `docs/tombstones/atlasctl/`.
- Any PR introducing a new `atlasctl` reference outside the tombstone path must fail policy review and repo checks.
- Python tooling documents are historical-only after cutoff and must live under the tombstone path.
