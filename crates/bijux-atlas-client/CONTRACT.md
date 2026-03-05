# bijux-atlas-client Contract

## Purpose

`bijux-atlas-client` provides a Rust SDK for Atlas HTTP API usage patterns.

## Scope

- Request/response client primitives in `src/`.
- Integration examples under `examples/`.
- SDK contract and behavior tests under `tests/`.

## Stability

- This crate is evolving and may change until the public API is declared stable.

## Inputs

- HTTP endpoint URL, request payloads, and client configuration.

## Outputs

- Typed response payloads and structured client errors.

## Invariants

- Request/response serialization stays deterministic for documented surfaces.
- API compatibility stays aligned with the published Atlas API contracts.

## Effects policy

- Network effects are limited to outbound API requests initiated by client calls.

## Error policy

- Transport and protocol failures return typed errors with actionable context.

## Versioning/stability

- Semver applies; breaking API changes require a major version increment.

## Tests expectations

- Contract tests verify deterministic payload handling and error mapping behavior.

## Dependencies allowed

- Runtime dependencies must be required for API transport, serialization, or tracing.

## Anti-patterns

- No hidden global mutable state.
- No implicit environment-variable configuration at call sites.

## Bench expectations

- Benches must measure client request/response hot paths with reproducible fixtures.

## Public API surface

- Public API is exposed through `src/lib.rs` and documented in crate-level docs.
