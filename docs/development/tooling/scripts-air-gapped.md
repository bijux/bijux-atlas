# Scripts Air-Gapped Mode

- Owner: `platform`
- Stability: `active`

## Goal

Run `atlasctl` in environments without outbound network access.

## Rules

- Use pinned lockfiles only (`requirements.lock.txt`), no ad-hoc dependency resolution.
- Run with `--no-network` for commands that do not require external calls.
- Unit tests must be network-denied by default; integration tests are explicitly marked.

## Recommended Setup

```bash
make atlasctl/internal/deps/sync
make scripts-test
```

## CI/Container Notes

- CI containers should pre-install dependencies from lockfiles.
- Avoid runtime `pip install` during gate execution.
- Prefer `PYTHONPATH=packages/atlasctl/src` or installed console entrypoint.
