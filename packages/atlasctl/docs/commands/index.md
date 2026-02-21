# Atlasctl Commands

Public command index (stable surface). Command wiring is registry-first via `src/atlasctl/cli/surface_registry.py`.

## Command Groups

- [Docs](groups/docs.md)
- [Configs](groups/configs.md)
- [Dev](groups/dev.md)
- [Ops](groups/ops.md)
- [Policies](groups/policies.md)
- [Internal](groups/internal.md)

## Stable Commands

- `check`
- `commands`
- `configs`
- `contracts`
- `doctor`
- `dev`
- `docs`
- `gates`
- `help`
- `inventory`
- `k8s`
- `layout`
- `lint`
- `list`
- `load`
- `make`
- `obs`
- `ops`
- `policies`
- `registry`
- `repo`
- `report`
- `run-id`
- `suite`
- `stack`
- `test`
- `version`

Internal commands are hidden from default help output.  
Use `atlasctl help --include-internal --json` to inspect them explicitly.

## Workflow

- Add new commands using `src/atlasctl/cli/surface_registry.py` as the single command catalog.
- Update snapshots only through `python -m atlasctl.cli gen goldens`.
- Follow `packages/atlasctl/docs/commands/new-command-workflow.md` when adding a command.
