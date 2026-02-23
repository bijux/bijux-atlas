# Dev Command Group

Commands mapped to `dev` effects policy.

## Commands

- `check`
- `contracts`
- `deps`
- `dev`
- `doctor`
- `gate`
- `gates`
- `gen`
- `install`
- `inventory`
- `layout`
- `lint`
- `list`
- `make`
- `owners`
- `packages`
- `registry`
- `release`
- `repo`
- `report`
- `run-id`
- `suite`
- `test`

## Examples

- `atlasctl check list --json`
- `atlasctl contracts list --json`
- `atlasctl deps lock --json`
- `atlasctl dev check -- domain repo`
- `atlasctl doctor --json`
- `atlasctl gate run --preset root --all --report json`
- `atlasctl gates --report json`
- `atlasctl gen goldens`
- `atlasctl install doctor --json`
- `atlasctl inventory --category all --format json`
- `atlasctl layout root-shape --json`
- `atlasctl lint run --report json`
- `atlasctl list checks --json`
- `atlasctl make lint --json`
- `atlasctl owners list`
- `atlasctl packages --json`
- `atlasctl registry list --json`
- `atlasctl release checklist --plan --json`
- `atlasctl repo stats --json`
- `atlasctl report summary --run-id local`
- `atlasctl run-id --prefix ci`
- `atlasctl suite run ci --json`
- `atlasctl test all --report json`
