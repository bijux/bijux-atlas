# Documentation operating model

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define the operating rules that keep published docs reviewable, owned, and enforceable.

## Reader spine

Mandatory entrypoints for all readers:

- `docs/index.md`
- `docs/start-here.md`
- `docs/product/index.md`
- `docs/operations/index.md`
- `docs/development/index.md`
- `docs/api/index.md`
- `docs/reference/index.md`
- `docs/governance/index.md`

## Review and ownership rules

- Stable pages require an explicit owner.
- Stable pages require `Last reviewed` or `Last verified against` metadata.
- Stable pages must be reviewed at least every `180` days.
- How-to and runbook pages must include verification guidance.
- Runbooks must include rollback and verify sections.

## Naming and structure rules

- File and directory naming standard: `kebab-case` only.
- Every top-level docs section must contain exactly one `index.md` entrypoint.
- Duplicate start pages are forbidden. Canonical start page is `docs/start-here.md`.
- Every document must include a short `reason to exist` statement.
- A page cannot mix tutorial and reference roles.

## Audience model

Only these audiences are valid:

- `user`
- `operator`
- `contributor`

## Page types

Only these page types are valid:

- `guide`
- `concept`
- `runbook`
- `reference`
- `policy`

## Cleanup cadence

- Quarterly cleanup is mandatory for the docs governance owner.
- Cleanup must review broken links, dead ends, duplicate titles, orphan pages, and stale generated artifacts.
- Cleanup output belongs in [Docs debt backlog](docs-debt-backlog.md).

## Glossary policy

- Canonical glossary location: `docs/glossary.md`.
- Section-local glossaries are not allowed.

## Redirect policy

- Legacy paths are redirected through `docs/redirects.json`.
- Redirects are required when previously linked paths move or are deleted.

## Deletion and quarantine policy

- Removals go through `_drafts/` only when immediate deletion is unsafe.
- Every quarantined file requires `moved_on`, `expiry_on`, and `owner` metadata.
- Quarantined files are excluded from navigation.

## Runtime accuracy rules

- Docs must not reference nonexistent CLI commands.
- Docs must not reference nonexistent make targets.
- Published nav must not expose `_generated/`, `_drafts/`, or `_nav/`.

## Review standard

A documentation change merges only if the updated state is clearer and more actionable than the prior state.
