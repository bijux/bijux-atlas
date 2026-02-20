# atlasctl

`atlasctl` is the canonical automation CLI for this repository.

## Global Flags

- `--json`: emit machine-readable JSON output.
- `--run-id <id>`: override run identifier.
- `--artifacts-dir <path>`: set artifacts/evidence root.
- `--cwd <path>`: run command from an explicit repo root.
- `--quiet`: suppress non-error logs.
- `--verbose`: enable verbose diagnostics.

## Core Commands

- `atlasctl self-check`
- `atlasctl version`
- `atlasctl explain <command>`

## Output Contract

- Schema: `configs/contracts/atlasctl-output.schema.json`
- Tool field: `tool=atlasctl`
- Commands must support `--help` via argparse command surface.
- Commands must support JSON output via global `--json`.

## Exit Codes

- `0`: ok
- `2`: user error
- `3`: contract violation
- `10`: prerequisite missing
- `20`: internal error
