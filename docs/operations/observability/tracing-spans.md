# Tracing Spans

Required span coverage:

- `http.request`
- query planning
- query execution
- ingest pipeline
- dataset loading
- cache lookup
- shard routing
- artifact loading
- cursor generation
- API serialization

Each request trace should show a complete causal path across these stages.
