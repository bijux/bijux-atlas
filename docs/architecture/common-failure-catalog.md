# Common failure catalog

- Owner: `architecture`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: map common architecture failure modes to operational runbooks.

| Failure mode | Primary signal | Runbook |
| --- | --- | --- |
| ingest validation failure | publish blocked | [Dataset corruption](../operations/runbooks/dataset-corruption.md) |
| registry merge conflict | release alias blocked | [Registry federation](../operations/runbooks/registry-federation.md) |
| serving-store outage | query/API failures | [Store outage](../operations/runbooks/store-outage.md) |
| load spike degradation | latency and timeout growth | [Traffic spike](../operations/runbooks/traffic-spike.md) |
| error-rate spike | elevated 5xx/contract errors | [SLO store backend error spike](../operations/runbooks/slo-store-backend-error-spike.md) |

## Next steps

- [Dataflow](dataflow.md)
- [Error model](error-model.md)
- [Reference errors](../reference/errors.md)
