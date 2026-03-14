# Data Flows

## Ingest

Dataset inputs are validated, transformed, and written to governed storage outputs.

## Store

Runtime and ingest outputs are cached or persisted through the configured storage backends.

## Query

Client requests flow through routing, policy, dependency calls, and cache lookup paths.

## Serve

Runtime responses, metrics, logs, and release evidence are emitted to operators and downstream systems.
