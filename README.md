# bijux-atlas

## Repository Layout

- `crates/`: Rust workspace crates.
- `configs/`: static configuration schemas and policy inputs.
- `ops/`: operations SSOT (`e2e`, `load`, `observability`, `openapi`, datasets).
- `docs/`: product/reference/contracts/operations/development docs.
- `scripts/`: automation entrypoints grouped by domain.
- `makefiles/`: target implementations included by root `Makefile`.

Compatibility shims retained at root:
- `e2e/`, `load/`, `observability/` as transition pointers/symlinks to `ops/*`.
- Root config symlinks (`deny.toml`, `audit-allowlist.toml`, `clippy.toml`, `rustfmt.toml`, `.vale.ini`, `.vale/`, `nextest.toml`).
- `bin/` keeps minimal bootstrap wrappers that delegate to `scripts/bin/*`.
- `ops/tool-versions.json`, `datasets`, and `fixtures` remain compatibility symlinks.

Operational policy:
- `ops/` is the canonical operations surface.
- `charts/` is packaging-only.
- Operational outputs are written under `artifacts/ops/<run-id>/`.
- `.idea/` is ignored and never committed.
- `target/` and `.DS_Store` must not be committed.

`.cargo/` remains at root intentionally because Cargo only resolves workspace config from root `.cargo/config.toml`.

## Single Entrypoint Policy

Use `make` targets as the runnable surface. Scripts are internal implementation details.

## Quick Commands

```bash
make help
make bootstrap
make doctor
make dev-ci
make docs-hardening
```
