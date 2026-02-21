# DEC-0002 Dependency Lock Strategy

## Decision

`requirements.in` and `requirements.lock.txt` are kept for deterministic `pip-tools` workflows, while tool/runtime configuration remains in `pyproject.toml`.

## Consequence

- `pyproject.toml` stays the configuration SSOT.
- Lock refresh remains explicit and reproducible via the deps command workflow.
