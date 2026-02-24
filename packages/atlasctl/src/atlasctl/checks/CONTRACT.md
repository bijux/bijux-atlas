# Atlasctl Checks Contract

## Canonical Tree

The `atlasctl.checks` package has one canonical shape:

- `checks/model.py`
- `checks/policy.py`
- `checks/runner.py`
- `checks/report.py`
- `checks/selectors.py`
- `checks/registry.py` as runtime SSOT surface
- `checks/domains/*.py` for check definitions
- `checks/tools/*.py` for reusable check helpers

## Prohibited Trees

These trees are not allowed in the target shape:

- `checks/layout/`
- `checks/repo/`
- `checks/registry/` package directory
- nested domain trees such as `checks/domains/*/*`

## Registry and Generated Artifacts

- Runtime check selection uses python registry APIs, not generated artifacts.
- `REGISTRY.generated.json` is a generated output only.
- `REGISTRY.toml` is generated-only when present.
- Runtime execution must not parse generated registry files as inputs.

## Runtime Boundaries

- The command layer uses one checks runner surface.
- Check implementations declare effects and owners.
- Check write roots stay under managed evidence roots.
