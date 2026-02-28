# Documentation Operating Model

Owner: `docs-governance`  
Status: `active`  
Effective date: `2026-02-28`

## Change Freeze Window

- Freeze starts: `2026-02-28`.
- Freeze ends: `2026-03-31`.
- During freeze, new pages are allowed only when they replace or consolidate existing content.

## Reader Spine

Mandatory entrypoints for all readers:

- `docs/index.md`
- `docs/start-here.md`
- `docs/product/index.md`
- `docs/operations/index.md`
- `docs/development/index.md`
- `docs/api/index.md`
- `docs/reference/index.md`
- `docs/governance/index.md`

## Hard Limits

- Top-level documentation directories: maximum `10`.
- Total authored pages: maximum `200`.
- Files directly under `docs/`: maximum `12`.

## Naming and Structure Rules

- File and directory naming standard: `kebab-case` only.
- Every top-level docs section must contain exactly one `index.md` entrypoint.
- Duplicate start pages are forbidden. Canonical start page is `docs/start-here.md`.
- Every document must include a short `reason to exist` statement.
- A page cannot mix tutorial and reference roles.

## Audience Model

Only these audiences are valid:

- `user`
- `operator`
- `contributor`

## Page Types

Only these page types are valid:

- `guide`
- `concept`
- `runbook`
- `reference`
- `policy`

## Canonical Product Statement

Use this exact one-paragraph statement when introducing Atlas:

`Atlas is the stable platform surface for operating, evolving, and consuming the bijux ecosystem through explicit contracts, predictable workflows, and verifiable runtime behavior.`

## Glossary Policy

- Canonical glossary location: `docs/glossary.md`.
- Section-local glossaries are not allowed.

## Redirect Policy

- Legacy paths are redirected through `docs/redirects.json`.
- Redirect entries must include `from`, `to`, `added_on`, and `expires_on`.

## Deletion and Quarantine Policy

- Removals go through `_drafts/` only when immediate deletion is unsafe.
- Every quarantined file requires `moved_on`, `expiry_on`, and `owner` metadata.
- Quarantined files are excluded from navigation.

## Runtime Accuracy Rules

- Docs must not reference nonexistent CLI commands.
- Docs must not reference nonexistent make targets.

## Review Standard

A documentation change merges only if the updated state is clearer and more actionable than the prior state.

## Ownership Model

- Each top-level section has exactly one accountable owner.
- Ownership must be listed in `docs/ownership.md`.

## Service Levels

- Urgent factual corrections: within `24h`.
- Normal corrections: within `72h`.
