# Dev Command Group

Commands in this group provide developer tooling and control-plane workflows.

## Commands

- `doctor`
- `inventory`
- `gates`
- `repo`
- `report`
- `suite`
- `lint`
- `contracts`
- `check`
- `test`
- `registry`
- `layout`
- `list`
- `dev`

## Examples

- `atlasctl doctor --json`
- `atlasctl inventory --category all --format json`
- `atlasctl gates run ci --report json`
- `atlasctl repo stats --json`
- `atlasctl report summary --run-id local`
- `atlasctl suite run ci --json`
- `atlasctl lint run --report json`
- `atlasctl contracts list --json`
- `atlasctl check list --json`
- `atlasctl test inventory --json`
- `atlasctl registry list --json`
- `atlasctl layout root-shape --json`
- `atlasctl list commands --json`
- `atlasctl dev check -- domain repo`
