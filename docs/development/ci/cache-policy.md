# CI Cache Policy

- Owner: `build-and-release`

## Rules

- Cache keys must include dependency lockfiles and toolchain versions.
- Rust cache keys must include:
  - `Cargo.lock`
  - rust toolchain version
- Python cache keys must include:
  - lockfile (or pinned dependency manifest)
  - python version
- Cache restores are performance hints only; CI correctness must not depend on cache hits.

## Why

Prevents stale caches from masking dependency drift and keeps builds deterministic.
