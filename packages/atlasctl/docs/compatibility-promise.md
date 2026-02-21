# Compatibility Promise

`bijux-atlas` command contracts are stable by default.

## Promise
- Patch releases preserve existing command names and required flags.
- Output schemas remain backward compatible for required fields.
- Exit code meanings remain stable for documented errors.

## Exceptions
- Experimental commands marked in `cli-stability.md` may change.
- Breaking changes require explicit migration notes and deprecation window.
- See `support-policy.md` for stable vs internal surfaces.
- See `schemas/breaking-change-policy.md` for output schema versioning rules.

## Verification
- `make scripts-check`
- `make scripts-test`
