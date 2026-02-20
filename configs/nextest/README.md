# Nextest Configs

Canonical test-runner profile configuration.

## Files

- `nextest.toml`
  - Consumer: `cargo nextest` through `make test` / `make dev-test-all`.

## Profiles

- Default profile is optimized for deterministic CI parity.
- Retry/failure behavior is declared in this file only.

## Policy

- Canonical configuration must stay in `configs/nextest/nextest.toml`.
- Tooling should reference this canonical path directly.

## Verification

```bash
make test
```
