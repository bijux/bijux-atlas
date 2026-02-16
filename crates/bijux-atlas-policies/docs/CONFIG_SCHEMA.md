# Config Schema Contract

Schema versioning rules:
- `schema_version` is required in both schema and config.
- Version must be numeric string.
- Bumps are monotonic and limited to step `+1` per rollout.

Compatibility:
- Unknown top-level keys are rejected.
- Nested policy blocks reject unknown keys.
- Missing required keys are invalid.
- Defaults are forbidden unless listed in `documented_defaults`.
