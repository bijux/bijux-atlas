# Nextest Configs

Canonical test-runner profile configuration.

## Files

- `nextest.toml`
  - Consumer: `cargo nextest` through `make test` / `make dev-test-all`.

## Profiles

- Default profile is optimized for deterministic CI parity.
- Retry/failure behavior is declared in this file only.

## Policy

- Root `nextest.toml` is a symlink shim only.
- Canonical configuration must stay in `configs/nextest/nextest.toml`.

## Verification

```bash
make test
```
