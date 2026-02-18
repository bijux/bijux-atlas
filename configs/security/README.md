# Security Configs

Canonical security policy configs consumed by CI and local gates.

## Files

- `deny.toml`
  - Consumer: `cargo-deny` via `make ci-deny` and `make ci-license-check`.
- `audit-allowlist.toml`
  - Consumer: `cargo-audit` allowlist handling and security review workflows.

## Policy

- Root `deny.toml` and `audit-allowlist.toml` are symlink shims only.
- Canonical content must live under `configs/security/`.

## Verification

```bash
make ci-deny
make ci-audit
```
