# Add A New Ops Action

Use this checklist when adding a public `atlasctl ops ...` action.

1. Add the parser entry in `commands/ops/runtime_modules/ops_runtime_parser.py`.
2. Implement behavior in the correct first-class area module (`commands/ops/<area>/...`) or runtime module.
3. Avoid direct `subprocess`; use `atlasctl.core.process` or `commands/ops/tools.py`.
4. Ensure reports include `inputs`, `config_hash`, `tool_versions`, `timings`, and `artifact_index`.
5. Add the action to `configs/ops/suites.json` (suite membership is required).
6. Update `docs/_generated/ops-actions.md` from the action inventory.
7. Run ops boundary checks and suite contract checks.

