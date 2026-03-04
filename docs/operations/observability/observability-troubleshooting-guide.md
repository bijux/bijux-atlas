# Observability Troubleshooting Guide

If metrics are missing:

- verify `/metrics` locally
- verify scrape target health
- check cardinality enforcement and scrape relabeling

If traces are missing:

- verify exporter mode and endpoint
- check sampling ratio
- inspect local trace fallback output

If logs are missing:

- verify JSON logging enabled
- verify ingestion parser for structured fields
- confirm redaction/sampling policy did not drop required events
