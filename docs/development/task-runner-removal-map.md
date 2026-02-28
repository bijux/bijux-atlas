# Control Plane Replacement Map

Legacy helper executables were removed. Use `bijux dev atlas` commands, or thin `make` wrappers that delegate to them:

- Contracts validation -> `cargo run -p bijux-dev-atlas -- configs validate`
- Policy reporting -> `cargo run -p bijux-dev-atlas -- policies report --format json`
- Docs lint -> `cargo run -p bijux-dev-atlas -- docs lint --format json`
- Fast CI gate -> `cargo run -p bijux-dev-atlas -- check run --suite ci_fast`
