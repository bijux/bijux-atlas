# ARCHITECTURE (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

## End-State Contract

- `bijux-dev-atlas` is the single control-plane crate; crate-internal modules are the only implementation surface for dev-atlas behavior.
- Effects live only in `adapters`; `core` remains pure and effect-free.
- `ports` is the only interface boundary between `core` and `adapters`.
- `commands` orchestrates flows and dependency wiring; it does not own core business logic.
- `cli` parses arguments and dispatches only.
- `model` is a leaf module for stable types and serialization contracts.
- `policies` is a leaf-oriented pure validation module.
- Root module budget is capped at 10 modules (`src/lib.rs`), and command/core source files follow LOC budgets (warning at 800, error at 1000 unless a stricter file-local rule applies).

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
- Runtime policy contracts live in the runtime crate (`bijux-atlas-policies`); dev-atlas governance policy loading/validation lives in `crate::policies::dev`.
- Dev policy source-of-truth paths are `ops/inventory/policies/dev-atlas-policy.json` and `ops/inventory/policies/dev-atlas-policy.schema.json`; command code must not duplicate these paths.
- Benchmarks and tests use the repository `artifacts/target` cache root (workspace `.cargo/config.toml`); bench code must not introduce custom `target/` writes.
- Benchmark groups and output names remain unique per bench file to preserve isolated performance histories.

## References

- Crate docs index: `crates/bijux-dev-atlas/docs/index.md`
- Repository docs index: `docs/index.md`
