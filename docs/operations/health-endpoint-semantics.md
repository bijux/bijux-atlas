# Health Endpoint Semantics

- `/healthz`: process is alive.
- `/readyz`: service is ready to serve user traffic.
- `/healthz/overload`: overload gate signal for protective throttling.

Operational checks are exercised via make targets:

```bash
$ make ops-smoke
$ make ops-drill-rate-limit
$ make ops-drill-memory-growth
```

Canonical targets: `ops-smoke`, `ops-drill-rate-limit`, `ops-drill-memory-growth`.
