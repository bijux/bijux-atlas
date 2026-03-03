# Domain Module Contract

- Owner: bijux-dev-atlas
- Stability: stable

## Purpose

Every domain is the single extension point for business rules. Domain code owns runnable
definitions and user-facing route registration, while `engine`, `registry`, `model`, and `runtime`
provide the shared execution substrate.

## Required Surface

Every domain module is expected to converge on this shape:

- `contracts()` returns the domain-owned contract specifications.
- `checks()` returns the domain-owned diagnostic check specifications.
- `routes()` returns the domain-owned CLI routes.

Those functions may be free functions or trait-backed adapters, but they must be the canonical
public entrypoints for the domain.

## Architectural Rules

1. Contract names are `snake_case`, stable, and avoid abbreviations.
2. Check names use action verbs: `validate_*`, `verify_*`, or `enforce_*`.
3. Domain code must not import other domain modules directly; cross-domain composition goes through
   `engine`, `registry`, or `model`.
4. `ui/` formats `model` structs only and must not import domain modules.
5. `runtime/` is the only layer allowed to touch filesystem, process, or environment effects
   directly.
6. Contracts and checks may access host effects only through the shared `World` trait.
7. `registry/` is the only source of truth for lists of runnable contracts and checks.
8. No nested `runtime_mod/` directories are allowed; each domain converges on `runtime.rs` or a
   `runtime/` subtree.
9. Shared helpers under `support/` must stay leaf utilities and cannot own business rules.
10. New work must not introduce `.inc.rs` or `.part` source files.
11. Every runnable declares `domain`, `mode`, `tags`, `cost_class`, and `stability`.
12. Every runnable declares deterministic evidence paths and artifact outputs.
13. Every runnable provides a short help string for discovery surfaces.
14. Exit codes come only from `model::exit_codes`.
15. `commands/` is temporary orchestration only and must not own validation rules.
16. Checks are diagnostics; contracts are repository invariants.

## Migration Note

The crate is still mid-migration. Existing modules that have not yet reached this shape are legacy
surfaces to be collapsed into the domain contract above; new work should follow this contract now.
