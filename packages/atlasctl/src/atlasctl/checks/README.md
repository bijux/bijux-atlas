# Atlasctl Checks

`atlasctl/checks` is the canonical and only home for check implementations.

## Domain Split

- `repo_shape`: repository root shape and deterministic structure checks.
- `makefiles`: makefile boundary and policy checks.
- `ops`: operations/runtime contract checks.
- `docs`: documentation integrity and surface checks.
- `observability`: observability contract adapters and checks.
- `artifacts`: generated artifact hygiene checks.

Legacy or transitional checks outside this tree are migration exceptions and must be removed.
