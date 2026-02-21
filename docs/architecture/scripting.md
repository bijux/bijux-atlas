# Scripting Architecture Contract

- Owner: `bijux-atlas-platform`
- Stability: `stable`

## What

Defines the end-state contract for repository automation and scripting.

## Core decisions

1. All automation logic lives in `packages/atlasctl`.
2. The single scripting CLI name is `bijux-atlas`.
3. User-facing library implementations (Python or other languages) belong under `packages/<name>` and must not host repo-ops automation.
4. Root `bin/` stays, but only for minimal execution shims (no business logic).
5. No direct `python scripts/...` invocations are allowed in docs/make surfaces.
6. No direct `bash scripts/...` invocations are allowed in docs/make surfaces.
7. Makefile automation must invoke the package CLI (`./bin/bijux-atlas` or `python -m atlasctl`), not ad-hoc Python paths.
8. Runtime evidence is non-committed and must write under ignored artifact roots.
9. Script-focused artifacts default to `artifacts/atlasctl/`; lane evidence remains under `artifacts/evidence/`.
10. Run IDs use `atlas-YYYYMMDD-HHMMSS-<gitsha>` and are only allowed in ignored artifact paths.
11. Deterministic generated outputs can be committed only when timestamp-free.
12. Runtime logs, lane reports, and run evidence must never be committed.

## Command taxonomy

Stable command families:
- `doctor`
- `report`
- `check`
- `gen`
- `ops`
- `docs`
- `ci`
- `release`

Current implementation maps these through `bijux-atlas` namespaces (for example `ops`, `docs`, `configs`, `policies`, `report`, `inventory`, `gates`), and aliases must remain documented.

## Internal command policy

- Internal commands may exist for maintainers but must be excluded from default user documentation.
- Internal commands must still honor run context, schema contracts, and output policies.

## Toolchain contract

- Python toolchain manager: `pip-tools` (SSOT in `python-toolchain.toml`).
- Test runner: `pytest`.
- Lint/format: `ruff`.
- Type checker: `mypy`.
- Packaging backend: `setuptools`.

## How to verify

```bash
make scripts-check
make gates-check
make docs/check
```

Expected output: policy and command-surface checks pass without direct legacy script invocations.
