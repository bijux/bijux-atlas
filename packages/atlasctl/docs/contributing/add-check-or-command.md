# Cookbook: Add A New Check Or Command

## Add A Check

1. Implement logic under `src/atlasctl/checks/<domain>/...`.
2. Register the check in `src/atlasctl/checks/<domain>/__init__.py` with a stable `CheckDef` id.
3. Add coverage in `tests/` and include schema/golden validation when output is JSON.
4. Add suite membership in `pyproject.toml` suites and update markers if needed.

## Add A Command

1. Add command metadata to `src/atlasctl/cli/surface_registry.py`.
2. Wire parser + dispatch through canonical CLI path.
3. Document command in `docs/commands/index.md` and the command-group page.
4. Add tests for help, JSON output, and contract behavior.
5. Refresh goldens only via `atlasctl gen goldens`.
