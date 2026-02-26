# ARCHITECTURE (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

## Internal Module Graph

Allowed dependency direction (strict):

- `cli` -> `commands`
- `commands` -> `adapters`, `core`, `model`, `policies`
- `core` -> `model`, `policies`, `ports`
- `adapters` -> `ports`, `model` (serialization helpers only when required)
- `policies` -> `model`
- `model` -> `std`/serde only
- `ports` -> `std` + shared error/trait contracts

Forbidden dependency examples:

- `core` -> `adapters`
- `model` -> `core`
- `policies` -> `adapters`
- `cli` -> host effects (`std::fs`, `Command`, env reads) beyond argument parsing needs

## Invariants

- Host effects are isolated to adapter implementations and explicitly tracked temporary exceptions.
- `cli` parses and dispatches only; command execution lives in `commands`.
- `core` owns check logic and report generation behavior.
- `model` and `policies` remain leaf modules with stable data contracts.

## References

- Crate docs index: `crates/bijux-dev-atlas/docs/INDEX.md`
- Repository docs index: `docs/index.md`
