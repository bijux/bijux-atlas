# Effect Boundary Map

## Modules
- `runtime/server_runtime_core.rs`: state/config/metrics structs and policy decisions.
- `runtime/server_runtime_app.rs`: app/router orchestration.
- `runtime/dataset_cache_manager_*.rs`: effectful cache/store operations.
- `effect_adapters/*_adapters.rs`: explicit effect adapters for filesystem/sqlite/clock/random.
- `http/*.rs`: protocol mapping and validation; effect calls must flow through runtime or adapter facades.

## Guardrails
- Raw filesystem calls in `http/` are forbidden except `http/effects_adapters.rs`.
- `*_support.rs` files remain support-only and cannot define route entrypoints.
