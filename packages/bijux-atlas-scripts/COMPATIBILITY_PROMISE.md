# Compatibility Promise

`bijux-atlas` command contracts are stable by default.

## Promise
- Patch releases preserve existing command names and required flags.
- Output schemas remain backward compatible for required fields.
- Exit code meanings remain stable for documented errors.

## Exceptions
- Experimental commands marked in `CLI_STABILITY.md` may change.
- Breaking changes require explicit migration notes and deprecation window.

## Verification
- `make scripts-check`
- `make scripts-test`
