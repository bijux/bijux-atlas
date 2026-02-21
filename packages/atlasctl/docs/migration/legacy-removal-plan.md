# Legacy Removal Plan

Policy: pre-1.0, legacy code must be deleted, not preserved.

## Milestones

1. Inventory and classification
- Run `atlasctl legacy inventory --report json`.
- Classify every `atlasctl/legacy/*` module as `delete`, `move`, or `rewrite`.

2. Runtime migration
- Move active docs runtime code from `legacy/docs_runtime*` into `commands/docs/*`.
- Move active ops runtime code from `legacy/ops_runtime*` into `commands/ops/*`.
- Move repo native checks from `legacy/repo_checks_native*` into `checks/repo/*`.

3. Delete compatibility package
- Delete `packages/atlasctl/src/atlasctl/legacy/`.
- Remove `commands/*/legacy.py` wrappers.
- Remove CLI wiring that imports `atlasctl.legacy.*`.

4. Guardrails and CI gates
- Keep import policy: no imports from `atlasctl.legacy` outside legacy package.
- Add reachability gate: zero importers for legacy modules.
- Add CI/test gate: legacy directory must be absent (or empty during transition only).

5. Docs cleanup
- Remove migration docs that describe active legacy compatibility paths.
- Keep one short historical note describing completion and removal date.
