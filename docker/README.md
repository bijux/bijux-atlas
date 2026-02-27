# Docker

Container build, validation, and release behavior is defined by executable gates.

## Intent

- Keep docker policy minimal, machine-readable, and enforceable.
- Keep docker docs limited to this file and `docker/CONTRACT.md`.
- Route all operational behavior through `bijux dev atlas docker ...` and make wrappers.

## Canonical Files

- `docker/README.md`
- `docker/CONTRACT.md`
- `docker/policy.json`
- `docker/images/runtime/Dockerfile`

Root `Dockerfile` is a shim symlink to the canonical runtime Dockerfile.

## Canonical Commands

```bash
make docker-validate
make docker-build
make docker-smoke
make docker-sbom
make docker-scan
make docker-lock
make docker-release
```

## Artifacts

Docker gate artifacts are written under:

`artifacts/<run_id>/...`

## Enforcement

Policy meaning lives in gates, not prose. See `docker/CONTRACT.md` for the contract-to-gate mapping.
