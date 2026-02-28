# Hot paths

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: identify hot runtime paths and bottleneck-sensitive behavior.

## Core hot paths

- Query planning and execution for common filtered requests.
- Serving-store index lookups for large datasets.
- API serialization and transport for high-request endpoints.

## Performance assumptions

- p50 latency remains within expected baseline under normal load.
- p95 latency remains stable under sustained concurrency.
- p99 latency degradation triggers explicit overload behavior and operator signal.

## Terminology used here

- p99: [Glossary](../glossary.md)
- Load: [Glossary](../glossary.md)

## Next steps

- [Performance model](performance-model.md)
- [Operations load testing](../operations/load/index.md)
- [Telemetry model](telemetry-model.md)
