# Docker Contract

Owner: `bijux-atlas-platform`

## Purpose

Define the canonical container build contract for `bijux-atlas`.

## Base Image Policy

- Runtime image base must be pin-safe and non-floating.
- `FROM ...:latest` is forbidden.
- Root `Dockerfile` is a symlink shim to `docker/images/runtime/Dockerfile` only.
- Dockerfiles must remain namespaced under `docker/images/`.

## Pinning Policy

- Build args that affect toolchain/runtime must have explicit defaults in Dockerfile.
- `RUST_VERSION` must stay pinned and reviewed.
- OCI labels must include build provenance fields.
- Runtime image size must satisfy `docker/contracts/image-size-budget.json`.

## Build Network Policy

- Build is allowed network access only for base image/package retrieval in builder stage.
- Runtime image must contain only compiled artifacts and required runtime metadata.

## SBOM and Scan Policy

- Container vulnerability scan is required in CI (`make docker-scan` / CI workflows).
- Scanner precedence: `trivy` then `grype`.
- Security policy source of truth remains `configs/security/*`.
- SBOM expectations are defined by `docker/contracts/sbom-policy.json`.

## Verification

```bash
make docker-check
make docker-scan
```
