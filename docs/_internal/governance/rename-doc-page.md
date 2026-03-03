# How To Rename A Doc Page

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@a10951a3e4e65b3b9be3bb67b16b4dc16a6d5287`
- Last changed: `2026-03-03`
- Reason to exist: define the safe rename workflow for documentation pages.

## Procedure

1. Move the page to the new stable name.
2. Add the old and new paths to `docs/redirects.json`.
3. Run `python3 scripts/docs/sync_redirects.py`.
4. Run `mkdocs build --strict`.
5. Update direct links in docs entrypoints before merging.
