# Toolchain

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: define canonical contributor tooling and quick troubleshooting.

## Core Tools

- Rust toolchain from `rust-toolchain.toml`
- Make targets for common local validation
- `bijux dev atlas` as control-plane entrypoint

## Troubleshooting

- If checks fail unexpectedly, run focused reproducer commands from [Debugging Locally](debugging-locally.md).
- If docs validation fails, verify canonical paths and section index links before editing generated artifacts.

## Verify Success

```bash
make check
make test
```

## What to Read Next

- [Control Plane](control-plane.md)
- [Debugging Locally](debugging-locally.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
