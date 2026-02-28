# Contracts Plugin Index

- Owner: `docs-governance`

## What

Index page for `contracts/plugin` documentation.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

List and maintain links to section pages in this directory.

## Examples

- `spec.md`
- `mode.md`

```bash
$ cargo run -p bijux-atlas-cli --bin bijux-atlas -- --bijux-plugin-metadata
```

Expected output shape:

```json
{
  "name": "atlas",
  "version": "1.0.0",
  "compatibility": {
    "umbrella_min": "1.0.0",
    "umbrella_max": "1.x"
  }
}
```

## Failure modes

Missing index links create orphan docs.

## How to verify

```bash
$ make docs
```

Expected output: docs build and docs-structure checks pass.

## See also

- [Docs Home](../../index.md)
- [Naming Standard](../../_style/naming-standard.md)
- [Terms Glossary](../../_style/terms-glossary.md)
