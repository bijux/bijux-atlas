# Atlasctl Checks

`atlasctl/checks` is the canonical and only home for check implementations.

## Root Contract

- Root python files in `atlasctl/checks/` are restricted to `__init__.py` only.
- Registry artifacts are allowed at root: `REGISTRY.toml` and `REGISTRY.generated.json`.
- Grouped check-domain folders live under `atlasctl/checks/domains/`, capped at `10` top-level modules.
- Root python module policy is stricter and requires exactly one (`__init__.py`).

## Domain Split

- `repo_shape`: repository root shape and deterministic structure checks.
- `makefiles`: makefile boundary and policy checks.
- `ops`: operations/runtime contract checks.
- `docs`: documentation integrity and surface checks.
- `observability`: observability contract adapters and checks.
- `artifacts`: generated artifact hygiene checks.

Legacy or transitional checks outside this tree are migration exceptions and must be removed.
