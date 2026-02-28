# Assets Index

- Owner: `docs-governance`

## What

Catalog of shared documentation assets (images and diagrams).

## Why

Centralized assets prevent duplication and broken references.

## Scope

Static files in `docs/_assets/`.

## Non-goals

No generated assets or binary artifacts from CI runs.

## Contracts

- Asset filenames must be kebab-case.
- Asset references must use docs-relative paths.

## Failure modes

Duplicate or untracked assets create drift and stale references.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass and links resolve.

## See also

- [Docs Home](../index.md)
- [Style Guide](../governance/style-guide.md)
- [Glossary](../glossary.md)
