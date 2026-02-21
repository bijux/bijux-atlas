# Legacy Removal Plan

Status: completed for release `0.1.0`.

Policy: pre-1.0, legacy code must be deleted, not preserved.

Release gate outcome:
- `packages/atlasctl/src/atlasctl/legacy/` absent.
- top-level `legacy`/`compat`/`migration` commands removed.
- `atlasctl internal legacy inventory` is the only legacy report entrypoint.
- suite/check gates enforce zero legacy paths and no deprecated command names.
