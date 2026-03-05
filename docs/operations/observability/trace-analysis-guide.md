# Trace Analysis Guide

1. Start with `observe traces verify` to ensure trace contract integrity.
2. Use `observe traces coverage` to validate required span coverage before incident analysis.
3. Inspect `observe traces topology` output to map request edges to subsystem spans.
4. Correlate slow or failing requests with `error.structured` and `query.execution` spans first.
