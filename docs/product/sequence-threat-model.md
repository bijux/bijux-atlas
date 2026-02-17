# Sequence Endpoint Threat Model

## Abuse vectors
- Very large regions to exhaust CPU/memory.
- High-RPS scraping of sequence ranges.
- Cache-bypass style request churn.

## Mitigations
- Early region validation and `422` policy rejection for oversized requests.
- Separate per-IP sequence token bucket.
- API key requirement for larger ranges.
- Response compression for large payloads.
- Deterministic ETag/Cache-Control for downstream caching.

## Residual risk
- Sustained many-small-region scraping remains possible and is controlled via rate limits and operational monitoring.
