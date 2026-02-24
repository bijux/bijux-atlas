# Docker SSOT

This is the single source of truth for container build/test/use in this repository.

## Canonical Commands

```bash
make docker-check
make docker-scan
make docker-release
```

## Build

- Canonical Dockerfile: `docker/images/runtime/Dockerfile`
- Root `Dockerfile` is shim only and must symlink to `docker/images/runtime/Dockerfile`.
- Build metadata labels are injected by `make docker-build`.

## Directory Layout

- `docker/images/`: image definitions (`runtime/` is canonical).
- `docker/contracts/`: policy contracts (allowlists, pinning, SBOM, size budget).
- `docker/`: container build and runtime assets.

## Runtime Smoke

`make docker-smoke` validates container binary public surface:
- `bijux-atlas --help`
- `bijux-atlas --version`
- `atlas-server --help`
- `atlas-server --version`

## Push

`make docker-push` is CI-only and fails when run locally without CI marker.
`make docker-release` is CI-only and runs `docker-check` + `docker-push`.

## Policy Links

- `docker/CONTRACT.md`
- `docs/operations/container.md`
- `configs/security/README.md`
