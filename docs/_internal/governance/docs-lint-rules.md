# Documentation lint rules

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: keep duplicate-topic, navigation, and markdown enforcement in one canonical lint policy.

## Duplicate Topic Rules

- Keep one canonical page per topic and use pointers instead of repeated prose.
- Canonical topic examples:
  - `docs/architecture/boundaries.md`
  - `docs/architecture/effects.md`
  - `docs/reference/contracts/plugins/spec.md`
  - `docs/reference/contracts/plugins/mode.md`
  - `docs/reference/contracts/compatibility.md`

## Navigation and structure rules

- No orphan published pages: every published page must be reachable from a canonical index.
- No duplicate nav titles for published pages.
- No `_generated`, `_drafts`, or `_nav` content in reader navigation.
- Keep generated JSON behind [Docs dashboard](docs-dashboard.md), not in reader guides.

## Observability Documentation Rules

- Canonical observability entrypoint is `docs/operations/observability/index.md`.
- Required core pages: `alerts.md`, `dashboards.md`, `tracing.md`, `slo-policy.md`.
- Terminology must use proper product names (`OpenTelemetry`, `Prometheus`, `Grafana`).

## Verification

```bash
make docs
```

Additional CI coverage:

- `mkdocs build --strict`
- docs-only workflow markdown lint and link checks

## Repository Lint Baseline

- `unsafe` is forbidden.
- `unwrap` and `expect` are denied in production code.
- `todo!()` is forbidden.
- `dbg!()`, `println!()`, and `eprintln!()` are denied by policy.
- `panic!()` is forbidden in library crates.

Lint source of truth:

- workspace lints in `Cargo.toml`
- `configs/rust/clippy.toml`
