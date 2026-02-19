# Docker SSOT

This is the single source of truth for container build/test/use in this repository.

## Canonical Commands

```bash
make docker-build
make docker-smoke
make docker-scan
make docker-push
```

## Build

- Canonical Dockerfile: `docker/Dockerfile`
- Root `Dockerfile` is shim only and must symlink to `docker/Dockerfile`.
- Build metadata labels are injected by `make docker-build`.

## Runtime Smoke

`make docker-smoke` validates container binary public surface:
- `bijux-atlas --help`
- `bijux-atlas --version`
- `atlas-server --help`
- `atlas-server --version`

## Push

`make docker-push` is CI-only and fails when run locally without CI marker.

## Policy Links

- `docker/CONTRACT.md`
- `docs/operations/container.md`
- `configs/security/README.md`
