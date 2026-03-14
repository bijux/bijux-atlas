# Tests for bijux-atlas-ingest

What belongs here:
- Contract tests for deterministic ingest outputs.
- Fixture matrix tests (tiny/minimal/edgecases/realistic smoke).
- Validation/anomaly behavior tests under strictness modes.

What does not belong here:
- Network-dependent tests.
- Store/server integration tests.

Fixture policy:
- No network access.
- Keep fixtures deterministic and versioned in-repo.
