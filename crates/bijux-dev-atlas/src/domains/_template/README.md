# Domain Template

This directory is intentionally not compiled. It documents the stable shape every domain should
converge on while the crate is refactored.

Expected files:

- `mod.rs`
- `contracts.rs`
- `checks.rs`
- `commands.rs`
- `runtime.rs`

The canonical public surface is:

- `contracts()`
- `checks()`
- `routes()`

See `crates/bijux-dev-atlas/docs/internal/domain-module-contract.md` for the full rule set.
