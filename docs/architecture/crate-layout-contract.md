# Crate Layout Contract

Each crate under `crates/` must keep a consistent top-level layout:

- `Cargo.toml`
- `src/`
- `tests/` (required, even if minimal)
- `INDEX.md`
- `docs/`
  - `docs/INDEX.md`
  - `docs/ARCHITECTURE.md`
  - `docs/PUBLIC_API.md`

Optional:

- `benches/` when benchmark coverage exists.

## README contract

Each crate `INDEX.md` must include and link:

- `Docs index`
- `Public API`
- `Effects & boundaries`
- `Telemetry`
- `Tests`
- `Benches`

This contract is enforced by `atlasctl docs crate-docs-contract-check --report text` and checked in CI.
