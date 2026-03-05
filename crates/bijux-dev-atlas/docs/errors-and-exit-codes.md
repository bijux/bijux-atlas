# Errors and Exit Codes

Canonical error and exit behavior for `bijux-dev-atlas`.

## Error Classes

- CLI usage / argument errors from command parsing and dispatch.
- Check execution/report errors from check and contract runs.
- Adapter capability denials (`fs_write`, `subprocess`, `git`, `network`).
- Domain contract errors from docs/configs/ops/security runtime commands.

## Process Exit Codes

- `0`: success
- `1`: policy or contract violation
- `2`: usage error
- `3`: execution or internal error

## Related

- `errors.md` (compatibility alias)
- `error-codes.md` (compatibility alias)
- `exit-codes.md` (compatibility alias)
