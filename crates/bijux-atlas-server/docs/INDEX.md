# Docs Index

- [Architecture](architecture.md)
- [Effects](effects.md)
- [Runtime](RUNTIME.md)
- [Telemetry](TELEMETRY.md)
- [Operations Runbook](OPERATIONS_RUNBOOK.md)
- [Caching](CACHING.md)
- [Failure Modes](FAILURE_MODES.md)
- [Kubernetes Ops](KUBERNETES.md)
- [Public API](public-api.md)
- [Tests](../tests/)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `docs/public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/public-api.md`, and add targeted tests.

- Central docs index: docs/index.md
- Crate README: ../README.md
