# Root Symlink Shims Policy

Owner: `platform`

## Policy

- Root symlinks are compatibility shims only.
- Canonical content must live under `configs/` (or `docker/` for `Dockerfile`).
- Root symlinks must match `configs/repo/symlink-allowlist.json`.
- New root symlinks require a documented approval token and a matching entry in `docs/development/symlinks.md`.

## Current Direction

- Prefer explicit tool config paths where supported.
- Keep only root symlinks that materially improve developer experience or are required by external tools.
- Root tool-config symlinks are removed; only `Dockerfile` is retained at root.

## Verification

```bash
make layout-check
make configs-check
```
