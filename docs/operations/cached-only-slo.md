# Cached-Only Mode SLO

## Mode Definition
- `ATLAS_CACHED_ONLY_MODE=true`
- Server serves only datasets already present and verified in local cache.
- No remote manifest/sqlite download attempts are allowed.

## Expected Behavior During Store Outage
- Cached datasets continue to serve with normal query SLO targets.
- Uncached datasets fail fast with `503` and a clear cache-miss message.
- No retry storms should be emitted toward store endpoints.

## Operational SLO
- For pinned/warmed datasets: maintain normal availability SLO.
- For cold datasets: fail fast within request timeout budget.
- Error budget accounting should treat uncached dataset requests as controlled degradation, not unexpected 5xx.

## See also

- `ops-ci`
