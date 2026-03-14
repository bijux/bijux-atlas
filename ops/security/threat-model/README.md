# Threat Model

This directory defines the Bijux Atlas threat model for the runtime, control-plane, release
artifacts, and governed operator workflows.

## Scope

- Runtime API serving, readiness, warmup, and dependency coordination
- Release evidence, SBOM, signing-adjacent integrity surfaces
- Control-plane command execution and generated reports
- Kubernetes profile, chart, and network policy governance

## Assumptions

- Operator and CI workflows use the governed control-plane entrypoints.
- Release evidence is the canonical artifact set for institutional review.
- Secrets should only appear in approved env or secret stores and must be redacted from evidence.

## Protected assets

See `assets.yaml` for the machine-readable inventory.
