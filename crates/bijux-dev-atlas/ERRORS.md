# Error Policy

This document defines error handling rules for `bijux-dev-atlas`.
Errors are structured, deterministic, and actionable.

## Rules

- Every surfaced error must include context and next-step guidance.
- Contract errors include contract id and failing evidence.
- Parsing errors include the failing field or path.
- I/O errors include the impacted path.
- CLI errors preserve stable wording where tests rely on snapshots.

## Sources

See `docs/error-codes.md` and `docs/contract.md` for canonical details.
