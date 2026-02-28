# Documentation Lint Rules

Owner: `docs-governance`  
Type: `policy`  
Audience: `contributor`  
Reason to exist: keep duplicate-topic and observability documentation checks in one canonical lint policy.

## Duplicate Topic Rules

- Keep one canonical page per topic and use pointers instead of repeated prose.
- Canonical topic examples:
  - `docs/architecture/boundaries.md`
  - `docs/architecture/effects.md`
  - `docs/reference/contracts/plugins/spec.md`
  - `docs/reference/contracts/plugins/mode.md`
  - `docs/reference/contracts/compatibility.md`

## Observability Documentation Rules

- Canonical observability entrypoint is `docs/operations/observability/index.md`.
- Required core pages: `alerts.md`, `dashboards.md`, `tracing.md`, `slo-policy.md`.
- Terminology must use proper product names (`OpenTelemetry`, `Prometheus`, `Grafana`).

## Verification

```bash
make docs
```

## Repository Lint Baseline

- `unsafe` is forbidden.
- `unwrap` and `expect` are denied in production code.
- `todo!()` is forbidden.
- `dbg!()`, `println!()`, and `eprintln!()` are denied by policy.
- `panic!()` is forbidden in library crates.

Lint source of truth:

- workspace lints in `Cargo.toml`
- `configs/rust/clippy.toml`
