# Docs Index

- [Architecture](ARCHITECTURE.md)
- [Effects](EFFECTS.md)
- [Runtime](RUNTIME.md)
- [Telemetry](TELEMETRY.md)
- [Caching](CACHING.md)
- [Failure Modes](FAILURE_MODES.md)
- [Kubernetes Ops](KUBERNETES.md)
- [Public API](PUBLIC_API.md)
- [Tests](../tests/)

## API stability

Public API is defined only by `docs/PUBLIC_API.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/PUBLIC_API.md`, and add targeted tests.

