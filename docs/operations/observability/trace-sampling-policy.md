# Trace Sampling Policy

- Owner: `bijux-atlas-operations`

## What

Defines trace sampling controls and required defaults.

## Why

Bounds telemetry overhead while retaining incident-debugging signal.

## Contracts

- SSOT knob: `telemetry.trace_sampling_per_10k` in `configs/policy/policy.json`.
- Pack profile sampling contract: `sampling_policy` in `configs/ops/observability-pack.json`.
- OTEL collector pipeline source: `ops/observe/pack/otel/config.yaml`.
- Validation: `crates/bijux-atlas-policies/src/validate.rs` rejects zero sampling.
- Runtime tracing remains opt-in via OTEL export wiring in server startup.
- Verification target: `ops-traces-check`, `ops-observability-pack-verify`.

## Failure Modes

Sampling set too low hides failure paths; set too high increases CPU and storage cost.

## How to verify

```bash
$ rg -n "trace_sampling_per_10k" configs/policy/policy.json configs/policy/policy.schema.json
$ make ops-traces-check
```

Expected output: policy key exists and trace checks pass when OTEL is enabled.

## See also

- [Tracing](tracing.md)
- [Config Keys Contract](../../contracts/config-keys.md)
- [Policy Schema](../../_generated/contracts/POLICY_SCHEMA.md)
