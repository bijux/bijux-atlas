# Environment Variables

This document is the SSOT for `atlasctl` runtime environment variables.

## Runtime

- `PYTHONPATH`: should include `packages/atlasctl/src` when running from source tree.
- `CI`: toggles CI-oriented defaults (for example default JSON/text output decisions).
- `ATLAS_NETWORK`: optional network mode override (`allow|forbid`) through command flags.

## Caches

- `XDG_CACHE_HOME`: base cache root used by atlasctl-managed tooling.
- `PYTHONPYCACHEPREFIX`: bytecode cache location.
- `MYPY_CACHE_DIR`: mypy cache location.
- `RUFF_CACHE_DIR`: ruff cache location.
- `PIP_CACHE_DIR`: pip cache location.
- `UV_CACHE_DIR`: uv cache location.

## Artifacts

- `BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT`: scripts artifact root used by atlasctl.
- `ATLASCTL_ARTIFACT_ROOT`: compatibility alias for artifact root.
