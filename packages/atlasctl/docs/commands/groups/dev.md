# Dev Command Group

Commands mapped to `dev` effects policy.

## Commands

- `check`
- `contracts`
- `dev`
- `doctor`
- `gates`
- `install`
- `inventory`
- `layout`
- `lint`
- `list`
- `make`
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
- `atlasctl dev check -- domain repo`
- `atlasctl doctor --json`
- `atlasctl gates --report json`
- `atlasctl install doctor --json`
- `atlasctl inventory --category all --format json`
- `atlasctl layout root-shape --json`
- `atlasctl lint run --report json`
- `atlasctl list checks --json`
- `atlasctl make lint --json`
- `atlasctl registry list --json`
- `atlasctl release checklist --plan --json`
- `atlasctl repo stats --json`
- `atlasctl report summary --run-id local`
- `atlasctl run-id --prefix ci`
- `atlasctl dev ci run --json`
- `atlasctl dev ci run --lane rust --fail-fast --json`
- `atlasctl dev ci run --lane docs --lane contracts --keep-going --json`
- `atlasctl dev fmt`
- `atlasctl dev lint`
- `atlasctl dev check`
- `atlasctl dev test`
- `atlasctl dev test --all`
- `atlasctl dev test --contracts`
- `atlasctl dev coverage`
- `atlasctl dev audit`
- `atlasctl test all --report json`
