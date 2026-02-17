# Load Suites

- Owner: `bijux-atlas-operations`

## What

Catalog of k6 scenarios and their intent.

## Why

Makes performance scenarios explicit for PR and nightly gates.

## Scope

Scenario files under `ops/e2e/k6/scenarios/*.json`.

## Non-goals

Does not duplicate k6 script implementation.

## Contracts

- `mixed.json`: baseline mixed traffic distribution.
- `spike.json`: burst overload behavior.
- `cold_start.json`: startup latency budget.
- `stampede.json`: thundering herd dataset requests.
- `store_outage.json`: store degradation behavior.
- `pod_churn.json`: restart churn behavior.
- `cheap_only_survival.json`: overload cheap-query survival.
- `response_size_guardrails.json`: payload guard enforcement.
- `multi_release.json`: cross-release query semantics.
- `multi_dataset_hotset.json`: hotset cache behavior.
- `large_dataset_simulation.json`: large dataset load profile.
- `sharded_fanout.json`: shard fanout caps.
- `redis_optional.json`: redis disabled fallback.
- `catalog_federated.json`: federated registry behavior.

## Budgets

- PR smoke suites must stay within short runtime budget.
- Nightly suites enforce full SLO thresholds from `configs/slo/slo.json`.

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
