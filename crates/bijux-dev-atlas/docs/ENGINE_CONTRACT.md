# Dev Control Plane Engine Contract

- Owner: bijux-dev-atlas
- Stability: evolving

## Purpose

Defines the execution boundary for the `bijux-dev-atlas` engine as the crate converges from split crates to internal modules.

## Layer Contract

- `core` owns deterministic evaluation, selection, validation, and report construction.
- `commands` owns command orchestration and capability wiring.
- `adapters` owns host effects (filesystem, subprocess, environment, network).
- `ports` defines effect interfaces consumed by `core`.

## Engine Inputs

- repository root
- selectors / filters
- capability flags
- adapter bundle implementing ports

## Engine Outputs

- stable report models under `model`
- deterministic machine-readable payloads
- explicit execution errors with stable taxonomy

## Invariants

- `core` must not depend on `adapters`
- host effects must be explicit and injectable
- deterministic ordering must be applied before output emission
- schema-bearing outputs must retain stable serialization shape
