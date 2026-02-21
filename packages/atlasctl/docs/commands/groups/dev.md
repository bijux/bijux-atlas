# Dev Command Group

Commands mapped to `dev` effects policy.

## Commands

- `check`
- `contracts`
- `dev`
- `doctor`
- `gates`
- `inventory`
- `layout`
- `lint`
- `list`
- `make`
- `registry`
- `repo`
- `report`
- `run-id`
- `suite`
- `test`

## Examples

- `atlasctl check list --json`
- `atlasctl contracts list --json`
- `atlasctl dev check -- domain repo`
- `atlasctl doctor --json`
- `atlasctl gates --report json`
- `atlasctl inventory --category all --format json`
- `atlasctl layout root-shape --json`
- `atlasctl lint run --report json`
- `atlasctl list checks --json`
- `atlasctl make lint --json`
- `atlasctl registry list --json`
- `atlasctl repo stats --json`
- `atlasctl report summary --run-id local`
- `atlasctl run-id --prefix ci`
- `atlasctl suite run ci --json`
- `atlasctl test all --report json`
