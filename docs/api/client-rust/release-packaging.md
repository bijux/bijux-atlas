# Rust Client Release Packaging

- Owner: `api-contracts`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`

## Packaging Checklist

- `cargo test -p bijux-atlas-client`
- `cargo check -p bijux-atlas-client --examples`
- `cargo bench -p bijux-atlas-client --no-run`
- Verify compatibility matrix and contract registry files.
