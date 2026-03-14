# Purity Policy

`bijux-atlas-model` is a pure domain crate.

Rules:
- No filesystem I/O.
- No network I/O.
- No process spawning.
- No clock/randomness dependencies.
- No dependencies on store/query/server crates.

Construction and normalization:
- Constructors/parsers are explicit (`parse_*`, `Type::parse`).
- No implicit normalization in strict constructors.
- Normalization helpers are separate and opt-in.
