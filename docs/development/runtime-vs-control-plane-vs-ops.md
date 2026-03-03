# Runtime vs control-plane vs ops

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: provide one-page role split across runtime, control-plane, and operations.

## Runtime

Owns ingest, query, API, server behavior and immutable release serving semantics.

## Control-plane

Owns contributor/CI validation orchestration, policy enforcement, and evidence outputs.

## Operations

Owns deployment, observability, incidents, drills, and production readiness workflows.

## Boundary Rule

Each layer owns its change domain; cross-layer shortcuts are treated as defects.

## What to Read Next

- [Architecture](../architecture/index.md)
- [Control-plane](../control-plane/index.md)
- [Operations](../operations/index.md)
