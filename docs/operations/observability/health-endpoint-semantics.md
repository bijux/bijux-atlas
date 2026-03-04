# Health Endpoint Semantics

- `/healthz` and `/health` report process liveliness and minimal service health.
- `/readyz` and `/ready` report traffic readiness.
- `/live` is a liveliness alias for environments that expect explicit live probes.
- `/healthz/overload` reports overload guard state, including breaker and shedding status.

Operational rule:

- Treat readiness as traffic gating.
- Treat health/liveness as process survival signal.
