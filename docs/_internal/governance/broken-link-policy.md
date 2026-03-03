# Broken Link Policy

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define response expectations for broken documentation links.

## Response Time

- Fix reader-facing broken links immediately in the next available docs change.
- Fix contributor-only broken links before the next docs publish.

## Repair Order

1. Restore or correct the destination when the page should exist.
2. Add a redirect when the path moved.
3. Remove or replace the source link when the target is intentionally gone.

## Validation

Run `mkdocs build --strict` after each broken-link fix. A docs change is incomplete if strict build
still reports broken links or invalid redirects.
