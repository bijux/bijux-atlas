# SLO Targets

Reference SLO targets used by perf and operations gates:

1. Availability: monthly successful request ratio >= 99.9% for non-overload valid requests.
2. Latency (steady state): p95 `/v1/genes` cheap class <= 120ms.
3. Latency (steady state): p95 `/v1/genes` medium class <= 250ms.
4. Latency (steady state): p95 `/v1/genes` heavy class <= 800ms.
5. Readiness: startup readiness achieved within configured warmup policy bounds.
6. Degradation mode: under overload, heavy-class shedding is allowed while cheap-class remains served.

These targets are operational defaults and can be tightened per environment policy.
