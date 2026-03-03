# How To Deprecate A Doc Page

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@a10951a3e4e65b3b9be3bb67b16b4dc16a6d5287`
- Last changed: `2026-03-03`
- Reason to exist: define the supported deprecation flow for documentation pages.

## Procedure

1. Add a deprecation notice to the page.
2. Add a redirect target in `docs/redirects.json` if the content moved.
3. Document the removal window in the change that introduced the deprecation.
4. Remove the page only after the deprecation window closes and links are updated.

## Minimum Window

Keep stable reader-facing pages in a deprecated state for at least one normal documentation review
cycle before removal, unless the page is actively harmful.
