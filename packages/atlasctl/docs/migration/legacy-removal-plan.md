# Legacy Removal Plan

Status: completed for release `0.1.0`.

Policy: pre-1.0, legacy code must be deleted, not preserved.

Release gate outcome:
- `packages/atlasctl/src/atlasctl/legacy/` absent.
- top-level `legacy`/`compat`/`migration` commands removed.
- `atlasctl internal legacy inventory` is the only legacy report entrypoint.
- suite/check gates enforce zero legacy paths and no deprecated command names.

## DEV/CI Legacy Target Plan

- Canonical DEV wrappers live in `makefiles/dev.mk` and delegate only to `./bin/atlasctl dev ...`.
- Canonical CI wrapper is `make ci` -> `./bin/atlasctl dev ci run`.
- `makefiles/cargo-dev.mk` is a deprecated compatibility shim; invoking its targets fails with migration guidance.
- Legacy make targets referenced by `configs/ops/nonroot-legacy-targets.txt` are banned in CI workflows after **2026-03-01** (`check_ci_legacy_target_cutoff.py`).
