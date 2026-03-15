# Operations Config Sources

This directory groups authored inputs for runtime operations, deployment policy, observability, and performance governance.

Current domains:
- `observability/` for logging and retention policy inputs.
- `ops/` for orchestration, runtime manifest, and operations surface inputs.
- `perf/` for benchmark and regression policy inputs.
- `slo/` for service-level objective declarations.
- `system/` for simulation and resilience inputs.

These files drive operator-facing workflows and operational verification. They stay separate from repository-maintainer policy and product-runtime surface definitions.
