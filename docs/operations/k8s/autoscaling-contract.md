# Autoscaling Contract

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines required HPA wiring, metrics pipeline prerequisites, and pass/fail evidence for autoscaling checks.

## Why

Prevents silent autoscaling misconfiguration and reduces flake in scale-up/scale-down verification.

## Scope

Applies to chart values, rendered templates, and k8s HPA verification checks in `ops/k8s/tests`.

## Non-goals

Does not guarantee exact replica counts at exact timestamps or guarantee production traffic shape in local tests.

## Contracts

- Values entrypoints: `values.hpa`, `values.metrics.customMetrics`, and `values.serviceMonitor`.
- `hpa.enabled=true` requires `metrics.customMetrics.enabled=true`.
- `hpa.cpuUtilization` must be within `1..100`.
- `hpa.behavior.scaleUp.policies` and `hpa.behavior.scaleDown.policies` must be non-empty.
- HPA checks require `metrics.k8s.io` API readiness; custom metrics checks require `custom.metrics.k8s.io`.
- HPA checks must prove both upscale intent (`desiredReplicas` change) and bounded downscale after load stops.
- HPA failures must dump HPA object/status/events into test artifacts.
- HPA `maxReplicas` must stay within profile safety caps from `configs/ops/hpa-safety-caps.json` to prevent runaway local scaling.

## Failure modes

- Missing metrics API/adapter causes preflight failure before scaling checks.
- Misconfigured HPA values fail during schema/template render.
- No observed desired replica change or no bounded downscale fails the HPA gate.
- Any values profile exceeding configured `maxReplicas` safety cap fails the HPA cap contract test.

## How to verify

```bash
make ops-k8s-template-tests
ATLAS_E2E_TEST=test_hpa.sh make ops-k8s-tests
ATLAS_E2E_TEST=test_hpa_misconfig_negative.sh make ops-k8s-tests
ATLAS_E2E_TEST=test_hpa_disabled_mode.sh make ops-k8s-tests
```

Expected output: template tests reject invalid HPA wiring and runtime tests produce scale evidence with deterministic pass/fail.

## See also

- [Kubernetes Operations Index](INDEX.md)
- [Values Schema](values-schema.md)
- [Packaging and Ops](packaging-and-ops.md)
