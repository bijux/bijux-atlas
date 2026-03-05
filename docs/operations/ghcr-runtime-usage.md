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
docker run --rm ghcr.io/bijux/bijux-atlas-server:v0.1.0 print-config-schema --json
docker run --rm ghcr.io/bijux/bijux-atlas-server:v0.1.0 self-check --json
```
