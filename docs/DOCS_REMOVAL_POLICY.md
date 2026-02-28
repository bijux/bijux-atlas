# Docs Removal Policy

- Owner: `docs-governance`
- Status: `stable`
- Audience: `contributors`

## What

Deleting docs is allowed when the page no longer serves the canonical docs spine, duplicates a stronger source, or has become stale.

## Why

The docs surface must stay small enough to review and accurate enough to trust. Removal is a maintenance tool, not a failure.

## Policy

- Prefer deleting or moving stale pages into `docs/_drafts/` over keeping low-signal pages in the indexed surface.
- Adding new stable docs requires explicit justification in code review and must fit within the growth budget contract.
- A replacement page must be linked when a deprecated page remains temporarily for redirect or migration reasons.

## How To Verify

```bash
make test
```

Expected output: docs governance contracts pass and the docs growth budget remains within policy.
