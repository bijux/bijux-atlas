# Telemetry Contract

Owner: `docs-governance`  
Type: `reference`  
Audience: `contributor`  
Reason to exist: provide one canonical contract for metrics and trace spans.

## Metrics Contract

- Source schema: `schemas/METRICS.json`
- Contract defines metric names and required label sets.
- Labels must avoid user-controlled high-cardinality values.

## Trace Spans Contract

- Source schema: `schemas/TRACE_SPANS.json`
- Contract defines span names and required attributes.

## Validation

```bash
make contracts
make contracts-docs
```
