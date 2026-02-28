# Stability Levels

- Owner: `docs-governance`

## Purpose

Define durable API and module stability language used across crate docs.

## Levels

- `incubating`: no compatibility promise; may change or be removed between patch releases.
- `provisional`: limited compatibility promise; additive changes preferred, breaking changes require explicit release note.
- `stable`: compatibility contract enforced; breaking changes require major version path/process.

## Usage Rules

- Every crate `docs/public-api.md` must reference this page.
- Avoid temporal labels (`phase`, `stage`, `iteration`) in stability statements.
- Use level + scope, for example: `public symbols: provisional`, `internal modules: incubating`.

## Example

- `Public API: provisional`
- `Internal module layout: incubating`
- `Error code set: stable`
