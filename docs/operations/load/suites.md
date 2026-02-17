# Load Suites

- Owner: `bijux-atlas-operations`

## What

Catalog of k6 scenarios and their intent.

## Why

Makes performance scenarios explicit for PR and nightly gates.

## Scope

Scenario files under `ops/load/scenarios/*.json`.

## Non-goals

Does not duplicate k6 script implementation.

## Contracts

- SSOT query set: `ops/load/queries/pinned-v1.json`
- Query freeze lock: `ops/load/queries/pinned-v1.lock`
- Suite manifest and budgets: `ops/load/suites/suites.json`
- `mixed.json`: baseline mixed traffic distribution.
- `spike.json`: burst overload behavior.
- `cold-start.json`: startup latency budget.
- `stampede.json`: thundering herd dataset requests.
- `store-outage.json`: store degradation behavior.
- `pod-churn.json`: restart churn behavior.
- `cheap-only-survival.json`: overload cheap-query survival.
- `response-size-guardrails.json`: payload guard enforcement.
- `multi-release.json`: cross-release query semantics.
- `multi-dataset-hotset.json`: hotset cache behavior.
- `large-dataset-simulation.json`: large dataset load profile.
- `sharded-fanout.json`: shard fanout caps.
- `redis-optional.json`: redis disabled fallback.
- `catalog-federated.json`: federated registry behavior.

## Budgets

- PR smoke suites must stay within short runtime budget.
- Nightly suites enforce full SLO thresholds from `configs/slo/slo.json`.
- `scripts/perf/score_k6.py` consumes both SLO policy and `suites.json` budgets.

## Failure modes

Scenario drift causes incomplete load coverage.

## How to verify

```bash
$ make e2e-perf
$ python3 scripts/perf/score_k6.py
```

Expected output: all configured suites produce results and pass policy thresholds.

## See also

- [Load Index](INDEX.md)
- [Load CI Policy](ci-policy.md)
- [SLO Targets](../../product/slo-targets.md)
- `ops-ci`
