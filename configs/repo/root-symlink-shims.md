# Root Symlink Shims Policy

Owner: `platform`

## Policy

- Root symlinks are compatibility shims only.
- Canonical content must live under `configs/` (or `docker/` for `Dockerfile`, `scripts/bin` for `bin`).
- New root symlinks require a documented approval token and a matching entry in `docs/development/symlinks.md`.

## Current Direction

- Prefer explicit tool config paths where supported (for example, `configs/nextest/nextest.toml` is now consumed directly by make targets).
- Keep only root symlinks that materially improve developer experience or are required by external tools.

## Verification

```bash
make layout-check
make configs-check
```
