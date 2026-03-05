# Running Atlas From GHCR

Use digest-pinned image references in all environments.

## Pull
```bash
docker pull ghcr.io/bijux/bijux-atlas-server:v0.1.0
```

## Run
```bash
docker run --rm -p 8080:8080 ghcr.io/bijux/bijux-atlas-server:v0.1.0 atlas serve
```

## Runtime Config Commands
```bash
bijux-dev-atlas runtime print-config-schema --format json
bijux-dev-atlas runtime self-check --format json
bijux-dev-atlas runtime explain-config-schema --format json
```
