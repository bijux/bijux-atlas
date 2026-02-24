# Xtask Removal Map

`xtask` was removed. Use `bijux dev atlas` commands (or thin `make` wrappers that delegate to them):

- `xtask check-contracts` -> `cargo run -p bijux-dev-atlas -- configs validate`
- `xtask scan-relaxations` -> `cargo run -p bijux-dev-atlas -- policies report --format json`
- `xtask docs-lint` -> `cargo run -p bijux-dev-atlas -- docs lint --format json`
- `xtask gates-fast` -> `cargo run -p bijux-dev-atlas -- check run --suite ci_fast`
