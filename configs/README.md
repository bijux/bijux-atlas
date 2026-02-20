# Configurations SSOT

Canonical home for repository configuration files and their consumers.

Rule:
- Root shims are allowed only when tooling requires root-path discovery.
- Canonical config content lives under `configs/`.
- Root tool-config symlinks are intentionally removed; tooling must use explicit config paths.

## Config Inventory

- `configs/rust/clippy.toml`
  - Consumed by: `cargo clippy`, `make ci-clippy`.
- `configs/rust/rustfmt.toml`
  - Consumed by: `cargo fmt`, `make ci-fmt`.
- `configs/nextest/nextest.toml`
  - Consumed by: `cargo nextest`, `make test`.
- `configs/security/deny.toml`
  - Consumed by: `cargo-deny`, `make ci-deny`, `make ci-license-check`.
- `configs/security/audit-allowlist.toml`
  - Consumed by: `cargo-audit` policy gates.
- `configs/docs/.vale.ini`
  - Consumed by: `vale` and `make docs`.
- `configs/docs/.vale/styles/**`
  - Consumed by: docs style/terminology lint.
- `configs/docs/requirements.txt`
  - Consumed by: docs env setup.
- `configs/docs/requirements.lock.txt`
  - Consumed by: reproducible docs runs (`make docs`, `make docs-serve`).
- `configs/policy/policy.schema.json`
  - Consumed by: `make policy-lint`, `make config-validate`.
- `configs/policy/policy.json`
  - Consumed by: runtime policy loading and policy lint gates.
- `configs/ops/env.schema.json`
  - Consumed by: `make ops-env-print`, env contract validation.
- `configs/ops/tool-versions.json`
  - Consumed by: `make ops-tools-check`, `make doctor`.
- `configs/ops/observability-pack.json`
  - Consumed by: observability pack profile/install/version checks, `make config-validate`.
- `configs/ops/artifacts-allowlist.txt`
  - Consumed by: layout/artifacts policy checks.
- `configs/ops/slo/classes.json`
  - Consumed by: SLO class contract checks and docs.
- `configs/ops/slo/sli.schema.json`
  - Consumed by: SLO contract validators.
- `configs/ops/slo/slo.schema.json`
  - Consumed by: `make ci-slo-config-validate`.
- `configs/ops/slo/slo.v1.json`
  - Consumed by: `make ci-slo-config-validate`, `make ci-slo-metrics-contract`.
- `configs/openapi/v1/openapi.generated.json`
  - Consumed by: OpenAPI drift checks and docs examples.
- `configs/openapi/v1/openapi.snapshot.json`
  - Consumed by: OpenAPI determinism/drift tests.
- `configs/perf/k6-thresholds.v1.json`
  - Consumed by: perf/load validation tooling.
- `configs/coverage/thresholds.toml`
  - Consumed by: coverage governance checks.
- `configs/slo/slo.json`
  - Consumed by: observability/SLO contract checks.

## Generated Registry

- `configs/config-key-registry.md`
  - Generated from `docs/contracts/CONFIG_KEYS.json`.
  - Regenerate with `make config-validate`.

## Related Docs

- `configs/rust/README.md`
- `configs/security/README.md`
- `configs/docs/README.md`
- `configs/nextest/README.md`
- `configs/coverage/README.md`
- `configs/repo/root-symlink-shims.md`
- `docs/development/tool-config-shims.md`
- `docs/development/config-versioning.md`

## Verification

```bash
make config-validate
make config-print
```
