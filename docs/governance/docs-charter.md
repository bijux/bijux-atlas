# Docs Charter

Owner: `bijux-atlas-docs`
Status: `active`
Audience: `contributors`

## Purpose

Atlas documentation is a reader-first system, not an archive. Every page must help a reader complete work, understand a stable contract, or execute an operational response.

## Freeze Window

No new stable documentation pages are allowed unless they replace an existing page or close a documented coverage gap.

## Reader Spine

Mandatory entrypoints:

- `docs/index.md`
- `docs/start-here.md`
- `docs/product/index.md`
- `docs/operations/index.md`
- `docs/development/index.md`
- `docs/reference/index.md`

## Structural Budgets

- Top-level documentation directories: maximum 10.
- Total markdown pages under `docs/`: maximum 200.
- Root files under `docs/`: maximum 12.

## Naming Standard

- All files and directories must use kebab-case.
- All section entrypoints must be named `index.md`.
- Uppercase file and directory names are forbidden.

## Authoring Rules

- Every doc requires a `Reason to exist` statement.
- A page cannot be both tutorial and reference.
- Duplicate `start-here` pages are forbidden.
- Any overview for a section must live in that section's `index.md`.

## Allowed Audiences

- `user`
- `operator`
- `contributor`

No additional audience class is allowed in stable docs.

## Allowed Page Types

- `guide`
- `concept`
- `runbook`
- `reference`
- `policy`

## Canonical Product Description

Atlas is a deterministic genomics serving platform that converts validated input artifacts into immutable, queryable releases with contract-governed APIs and operator workflows.

## Glossary Strategy

- Glossary source of truth: `docs/glossary.md`.
- Mini-glossaries inside section pages are forbidden.

## Redirect Strategy

- Redirect mappings live in `docs/redirects.json`.
- Redirects are temporary and must have explicit expiry handling in governance review.

## Deletion and Quarantine

- Candidate removals move to `docs/_drafts/` with an explicit expiry date.
- Expired drafts are deleted.
- Unlinked pages are removed or quarantined, not retained indefinitely.

## Accuracy Guards

- Docs must not reference CLI commands that do not exist.
- Docs must not reference make targets that do not exist.

## Review Bar

A docs change merges only if the resulting state is clearer than before for the intended audience.

## Ownership Model

Each top-level docs section has exactly one accountable owner.

See: `docs/ownership.md`.
