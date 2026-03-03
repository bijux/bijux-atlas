# Toolchain

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
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

- [Control-plane](../control-plane/index.md)
- [Debugging Locally](debugging-locally.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
