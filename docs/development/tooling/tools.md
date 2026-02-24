# Tooling Directory Intent Map

## Top-level directories and why they exist

- `crates/bijux-dev-atlas/`: canonical repository tooling CLI and automation command surface.
- `configs/`: source-of-truth configuration and policy inputs used by tooling and checks.
- `ops/`: operational runbooks, scripts, schemas, and deployment/test harnesses.
- `docs/`: human documentation and generated docs surfaces.
- `makefiles/`: delegated task entrypoints for repository workflows and checks.
- `crates/`: Rust runtime crates and contract implementations.
- `artifacts/`: generated evidence/output roots (non-source runtime artifacts).
- `docker/`: container build contracts and runtime image definitions.

## Notes

- `bijux dev atlas` is the Rust control-plane entrypoint for dev governance; package CLIs live under `crates/`.
- `make` targets are orchestration shims, not business-logic hosts.
