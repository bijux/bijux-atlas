# Compatibility Index

- Owner: `bijux-atlas-contracts`

## What
Index page for compatibility semantics across API, artifacts, cursors, and ecosystem integration.

## Why
A single compatibility section reduces discovery time and avoids duplicated compatibility policy text.

## Scope
Covers reference-level compatibility guidance. Contract-level guarantees remain in `docs/contracts/compatibility.md`.

## Non-goals
Does not duplicate contract registries.

## Contracts
- Compatibility constraints in this section must link back to contract SSOT pages.
- Policy pages in this section must use `*-policy.md` naming.

## Failure modes
Compatibility notes outside this section create inconsistent upgrade guidance.

## How to verify
```bash
make docs
```

## See also
- [Contracts Compatibility](../../contracts/compatibility.md)
- [Product Compatibility Promise](../../product/compatibility-promise.md)
- [Evolution Index](../evolution/INDEX.md)
