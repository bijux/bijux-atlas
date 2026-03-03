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
3. Ensure the redirect resolves through `docs/_internal/governance/redirect-registry.json` with owner and reason.
4. If the redirect is temporary, declare `expiresOn`.
5. Run `bijux-dev-atlas docs redirects sync --allow-write`.
6. Run `mkdocs build --strict`.
7. Update direct links in docs entrypoints before merging.
