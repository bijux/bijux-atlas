# Configurations SSOT

This directory is the canonical home for repository configuration files.

Rule:
- Root shims are allowed only when tooling requires root-path discovery.
- Canonical config content lives under `configs/`.

## Layout

- `configs/rust/`
  - `clippy.toml` (lint policy; root shim: `clippy.toml`)
  - `rustfmt.toml` (format policy; root shim: `rustfmt.toml`)
- `configs/nextest/`
  - `nextest.toml` (test runner profiles; root shim: `nextest.toml`)
- `configs/security/`
  - `deny.toml` (cargo-deny policy; root shim: `deny.toml`)
  - `audit-allowlist.toml` (audit exceptions; root shim: `audit-allowlist.toml`)
- `configs/docs/`
  - `.vale.ini` and `.vale/` ruleset (root shims: `.vale.ini`, `.vale`)
  - `requirements.txt` is intentionally under `ops/docs/requirements.txt` for docs runtime tooling.
- `configs/policy/`
  - policy runtime schema and default policy
- `configs/slo/`
  - SLO SSOT
- `configs/perf/`
  - performance thresholds
- `configs/coverage/`
  - coverage thresholds
- `configs/ops/`
  - tool lockfiles used by ops checks (`tool-versions.json`)
  - environment contract schema for ops targets (`env.schema.json`)

## Verification

```bash
make layout-check
make ops-tools-check
```
