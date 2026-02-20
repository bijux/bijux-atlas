# Scripts Air-Gapped Mode

- Owner: `platform`
- Stability: `active`

## Goal

Run `bijux-atlas-scripts` in environments without outbound network access.

## Rules

- Use pinned lockfiles only (`requirements.lock.txt`), no ad-hoc dependency resolution.
- Run with `--no-network` for commands that do not require external calls.
- Unit tests must be network-denied by default; integration tests are explicitly marked.

## Recommended Setup

```bash
make scripts-venv
make scripts-install
make scripts-test
BIJUX_SCRIPTS_TEST_NO_NETWORK=1 make scripts-test-hermetic
```

## CI/Container Notes

- CI containers should pre-install dependencies from lockfiles.
- Avoid runtime `pip install` during gate execution.
- Prefer `PYTHONPATH=tools/bijux-atlas-scripts/src` or installed console entrypoint.
