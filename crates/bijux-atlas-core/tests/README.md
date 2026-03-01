# Tests for bijux-atlas-core

What belongs here:

- Integration tests for public behavior of this crate.
- Policy/guardrail tests relevant to this crate boundary.
- Determinism tests for canonical bytes, hashes, and stable ordering.
- Export-surface contract checks against `docs/public-api.md`.

What does not belong here:

- End-to-end cross-service tests (keep at workspace/system level later).
- Golden/snapshot tests that depend on non-deterministic environment data.
- Tests that require filesystem/network/process I/O in core logic.
