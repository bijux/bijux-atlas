# Rust Client Architecture

- Owner: `api-contracts`
- Type: `reference`
- Audience: `user`
- Stability: `stable`

## Layers

- `config`: client runtime configuration and validation.
- `request`: endpoint and query parameter builder.
- `client`: HTTP transport, retry, timeout, and response decoding.
- `query`: dataset query model and filter/projection helpers.
- `pagination`: page shape and cursor helpers.
- `metrics`: request timing and success/failure observation hook.
- `tracing`: request and trace identifier propagation model.
- `error`: classification-first error model for callers.

## Design Goals

- Keep network interactions deterministic and observable.
- Expose compatibility-safe query wrappers.
- Provide integration hooks without coupling to a concrete telemetry stack.
