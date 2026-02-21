# Atlasctl Commands

Public command index (stable surface). Command wiring is registry-first via `src/atlasctl/cli/registry.py`.

## Stable Commands

- `check`
- `commands`
- `configs`
- `contracts`
- `doctor`
- `docs`
- `gates`
- `help`
- `inventory`
- `k8s`
- `layout`
- `lint`
- `load`
- `obs`
- `ops`
- `policies`
- `registry`
- `repo`
- `report`
- `suite`
- `stack`
- `test`
- `version`

Internal and migration-only commands (including `legacy` and `compat`) are hidden from default help output.  
Use `atlasctl help --include-internal --json` to inspect them explicitly.

## Workflow

- Add new commands using `src/atlasctl/cli/registry.py` as the single command catalog.
- Update snapshots only through `python -m atlasctl.cli gen goldens`.
- Follow `packages/atlasctl/docs/commands/new-command-workflow.md` when adding a command.
