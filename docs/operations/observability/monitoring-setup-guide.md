# Monitoring Setup Guide

Setup order:

1. Prometheus scrape target for `/metrics`.
2. Log pipeline for structured JSON logs.
3. Trace exporter endpoint (`otlp` or `jaeger`).
4. Dashboard import and datasource binding.

Verification:

- metrics visible in Prometheus
- traces visible in tracing backend
- alert rules loaded without parse errors
