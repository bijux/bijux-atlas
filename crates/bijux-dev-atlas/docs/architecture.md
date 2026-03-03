# ARCHITECTURE (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

## End-State Contract

- `bijux-dev-atlas` is the single control-plane crate; crate-internal modules are the only implementation surface for dev-atlas behavior.
- Domains are the only extension axis and converge on `contracts()`, `checks()`, and `routes()`.
- `app` owns binary entry wiring only.
- `cli` parses arguments and dispatches only.
- `engine` owns generic runner, selection, and reporting behavior.
- `registry` is the source of truth for lists of runnable contracts and checks.
- `runtime` is the only boundary allowed to touch filesystem, process, or environment effects.
- `model` is a leaf module for stable types and serialization contracts.
- `support` remains leaf-only helper code and cannot own business rules.
- `commands` is temporary orchestration glue during migration and must not own validation logic.
- New work must not introduce `.inc.rs` or `.part` source files.
- Root module budget is capped at 10 modules (`src/lib.rs`), and command/core source files follow LOC budgets (warning at 4500, error at 5000 unless a stricter file-local rule applies).

## Internal Module Graph

Allowed dependency direction (strict):

- `app` -> `cli`
- `cli` -> `commands`, `domains`
- `domains` -> `engine`, `registry`, `model`, `runtime`
- `commands` -> `domains`, `engine`, `registry`, `model`, `runtime`
- `engine` -> `model`, `registry`, `runtime`
- `registry` -> `model`
- `runtime` -> `ports`, `model`
- `policies` -> `model`
- `model` -> `std`/serde only
- `ports` -> `std` + shared error/trait contracts

Forbidden dependency examples:

- `domains/*` -> `domains/*` (cross-domain imports)
- `ui/*` -> `domains/*`
- `model` -> `runtime`
- `policies` -> `runtime`
- `cli` -> host effects (`std::fs`, `Command`, env reads) beyond argument parsing needs
- `contracts` / `checks` -> host effects without `World`

## Invariants

- Host effects are isolated to adapter implementations and explicitly tracked temporary exceptions.
- `cli` parses and dispatches only; command execution lives in `commands`.
- `core` owns check logic and report generation behavior.
- `model` and `policies` remain leaf modules with stable data contracts.
- Runtime policy contracts live in the runtime crate (`bijux-atlas-policies`); dev-atlas governance policy loading/validation lives in `crate::policies::dev`.
- Dev policy source-of-truth paths are `ops/inventory/policies/dev-atlas-policy.json` and `ops/inventory/policies/dev-atlas-policy.schema.json`; command code must not duplicate these paths.
- Benchmarks and tests use the repository `artifacts/target` cache root (workspace `.cargo/config.toml`); bench code must not introduce custom `target/` writes.
- Benchmark groups and output names remain unique per bench file to preserve isolated performance histories.

## Naming Rules

- Contract names use `snake_case`, stay stable, and avoid abbreviations.
- Check names use action verbs: `validate_*`, `verify_*`, or `enforce_*`.
- Every runnable declares `domain`, `mode`, `tags`, `cost_class`, `stability`, deterministic
  evidence paths, artifact outputs, and a short help string.
- Exit codes come from `model::exit_codes` only.
- Checks are diagnostics; contracts are repository invariants.

## References

- Crate docs index: `crates/bijux-dev-atlas/docs/index.md`
- Repository docs index: `docs/index.md`
- Domain module contract: `crates/bijux-dev-atlas/docs/internal/domain-module-contract.md`
