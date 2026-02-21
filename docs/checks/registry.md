# Checks Registry

`packages/atlasctl/src/atlasctl/checks/REGISTRY.toml` is the SSOT for check registration.

## Contract

- Every check must have one registry row.
- Every registry row must have an implementation.
- IDs must use `checks_<domain>_<area>_<name>`.
- Rows are sorted by `id`.
- `REGISTRY.generated.json` is generated from TOML and used at runtime.

## Required Fields

- `id`
- `domain`
- `area`
- `owner`
- `speed` (`fast|slow`)
- `groups`
- `timeout_ms`
- `module`
- `callable`
- `description`

## Commands

- Regenerate: `./bin/atlasctl gen checks-registry`
- Drift check: `./bin/atlasctl check checks-registry-drift`
- Migration inventory: `./bin/atlasctl migrate checks-registry`
- Browse:
  - `./bin/atlasctl checks list --json`
  - `./bin/atlasctl checks tree --json`
  - `./bin/atlasctl checks owners --json`
  - `./bin/atlasctl checks groups --json`
  - `./bin/atlasctl checks slow --json`
  - `./bin/atlasctl checks explain <id> --json`

## Check Run Selection

- `./bin/atlasctl check run --id <id>`
- `./bin/atlasctl check run -k <substring>`
- `./bin/atlasctl check run --domain repo`
- `./bin/atlasctl check run --group repo`
- `./bin/atlasctl check run --slow`
- `./bin/atlasctl check run --fast`
