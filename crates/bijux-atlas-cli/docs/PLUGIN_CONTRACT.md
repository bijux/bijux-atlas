# Plugin Contract

`bijux-atlas` is a Bijux plugin binary. Umbrella dispatches as:

- `bijux atlas ...` -> `bijux-atlas ...`

Required flags and behavior:

- `--bijux-plugin-metadata` prints plugin metadata JSON.
- `--json` enables machine-readable command output.
- `--quiet`, `--verbose`, `--trace` control logging behavior.

Metadata JSON required fields:

- `name`
- `version`
- `compatible_umbrella`
- `build_hash`

Exit code policy follows shared Bijux exit code registry in `bijux-atlas-core`.
