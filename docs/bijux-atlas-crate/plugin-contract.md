# Plugin Contract

`bijux-atlas` is a Bijux plugin binary. Umbrella dispatches as:

- `bijux atlas ...` -> `bijux-atlas ...`

Required flags and behavior:

- `--bijux-plugin-metadata` prints plugin metadata JSON.
- `--json` enables machine-readable command output.
- `--quiet`, `--verbose`, `--trace` control logging behavior.

Metadata JSON required fields:

- `schema_version`
- `name`
- `version`
- `compatible_umbrella`
- `compatible_umbrella_min`
- `compatible_umbrella_max_exclusive`
- `build_hash`

Exit code policy follows the shared Bijux exit code registry in `bijux_atlas::contracts::errors`.

Snapshot artifacts:

- `docs/PLUGIN_METADATA_SNAPSHOT.json`
- `tests/plugin_conformance.rs` enforces metadata and doc contract stability.
