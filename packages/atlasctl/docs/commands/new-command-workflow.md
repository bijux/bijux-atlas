# How To Add A New Command

1. Add the command to `packages/atlasctl/src/atlasctl/cli/surface_registry.py` with complete metadata:
   `owner`, `doc_link`, `purpose`, `examples`, `touches`, `tools`, and effect metadata.
2. Add parser wiring in `packages/atlasctl/src/atlasctl/cli/main.py` (and command module parser if domain-specific).
3. Add command dispatch in `packages/atlasctl/src/atlasctl/cli/dispatch.py`.
4. Add/extend tests for command behavior and JSON schema validation.
5. Update `packages/atlasctl/docs/commands/index.md` for stable commands.
6. Regenerate goldens only through:
   `python -m atlasctl.cli gen goldens`
7. Run command-surface checks:
   `python -m atlasctl.cli commands lint --json`
   and `python -m atlasctl.cli commands compat-check --json`.
