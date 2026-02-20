# Tooling Directory Intent Map

## Top-level directories and why they exist

- `packages/atlasctl/`: canonical repository tooling CLI and automation command surface.
- `configs/`: source-of-truth configuration and policy inputs used by tooling and checks.
- `ops/`: operational runbooks, scripts, schemas, and deployment/test harnesses.
- `docs/`: human documentation and generated docs surfaces.
- `makefiles/`: delegated task entrypoints that call `atlasctl`.
- `crates/`: Rust runtime crates and contract implementations.
- `artifacts/`: generated evidence/output roots (non-source runtime artifacts).
- `docker/`: container build contracts and runtime image definitions.
- `bin/`: stable executable entrypoint wrappers.

## Notes

- `atlasctl` is the SSOT tooling entrypoint.
- `make` targets are orchestration shims, not business-logic hosts.
