# Effects Policy

`bijux-atlas-model` must remain pure domain modeling code.

Allowed:
- Parsing/validation and deterministic formatting.
- Pure transformations between model values.

Forbidden:
- Filesystem/network/process I/O.
- Runtime store/database integrations.
