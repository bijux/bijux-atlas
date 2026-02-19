# bijux-atlas

## Repository Layout

- `crates/`: Rust workspace crates.
- `configs/`: static configuration schemas and policy inputs.
- `ops/`: operations SSOT (`e2e`, `load`, `observability`, `openapi`, datasets).
- `docs/`: product/reference/contracts/operations/development docs.
- `scripts/`: automation entrypoints grouped by domain.
- `makefiles/`: target implementations included by root `Makefile`.
- `docker/`: canonical container build surface and policy.

Compatibility shims retained at root:
- Root config symlinks (`deny.toml`, `audit-allowlist.toml`, `clippy.toml`, `rustfmt.toml`, `.vale.ini`, `.vale/`, `nextest.toml`).
- `bin/` keeps minimal bootstrap wrappers that delegate to `scripts/bin/*`.

Operational policy:
- `ops/` is the canonical operations surface.
- Legacy root aliases (`charts`, `e2e`, `load`, `observability`, `datasets`, `fixtures`) are forbidden.
- Operational outputs are written under `artifacts/ops/<run-id>/`.
- `.idea/` is ignored and never committed.
- `target/` and `.DS_Store` must not be committed.
- Local editors may create noise files; CI gates only fail when noise is tracked or CI workspace hygiene is violated.

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

Container references:
- Docker SSOT: `docker/README.md`
- Container/Kubernetes relation: `docs/operations/container.md`


## Quickstart (Make Targets)

```bash
make bootstrap
make doctor
make ops-up
make ops-deploy
make ops-warm
make ops-smoke
```
