# Trace Span Naming Conventions

- Use dot-separated lower_snake_case names.
- Prefix names by subsystem intent: `runtime.*`, `http.*`, `query.*`, `ingest.*`, `artifact.*`, `registry.*`, `configuration.*`, `lifecycle.*`, `error.*`.
- Span names are stability-governed and must not be repurposed.
