# Purity Contract

`bijux-atlas-core` is a deterministic utility crate.

Hard bans:
- No network I/O
- No filesystem I/O in core logic
- No process spawning
- No wall-clock behavior in deterministic paths
- No randomness (`rand` dependency/feature is forbidden)

Dependency policy:
- `serde_json` is allowed only behind core feature `serde`.
- Canonicalization helpers are the only allowed users of `serde_json`.
