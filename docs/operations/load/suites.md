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
- Suite schema: `ops/load/contracts/suite-schema.json`
- Suite manifest validator: `scripts/perf/validate_suite_manifest.py`
- Suite naming convention: `kebab-case`, unique, deterministic.
- Required suite fields: `purpose`, `kind`, `scenario|script`, `thresholds`, `expected_metrics`, `must_pass`.
- Result contract: `ops/load/contracts/result-schema.json`
- `mixed.json`: baseline mixed traffic distribution.
- `spike.json`: burst overload behavior.
- `cold-start.json`: startup latency budget.
- `stampede.json`: thundering herd dataset requests.
- `store-outage-mid-spike.json`: store degradation behavior during spike.
- `pod-churn.json`: restart churn behavior.
- `cheap-only-survival.json`: overload cheap-query survival.
- `response-size-abuse.json`: payload guard enforcement.
- `multi-release.json`: cross-release query semantics.
- `diff-heavy.json`: diff endpoint heavy workload profile.
- `mixed-gene-sequence.json`: combined gene summary and sequence request mix.
- `load-under-rollout.json`: load while deployment rollout executes.
- `load-under-rollback.json`: load while deployment rollback executes.
- `multi-dataset-hotset.json`: hotset cache behavior.
- `large-dataset-simulation.json`: large dataset load profile.
- `sharded-fanout.json`: shard fanout caps.
- `soak-30m.json`: long soak with memory growth checks.
- `redis-optional.json`: redis disabled fallback.
- `ops/load/experiments/`: non-gating experiment space with strict promotion policy.
- `catalog-federated.json`: federated registry behavior.

## Budgets

- PR smoke suites must stay within short runtime budget.
- Nightly suites enforce full SLO thresholds from `configs/slo/slo.json`.
- `scripts/perf/score_k6.py` consumes both SLO policy and `suites.json` budgets.

## Failure modes

Scenario drift causes incomplete load coverage.

## How to verify

```bash
$ make ops-load-smoke
$ make ops-load-full
$ python3 scripts/perf/score_k6.py
$ python3 scripts/perf/validate_results.py artifacts/perf/results
$ python3 ops/load/reports/generate.py
```

Expected output: all configured suites produce results and pass policy thresholds.

## See also

- [Load Index](INDEX.md)
- [Load CI Policy](ci-policy.md)
- [SLO Targets](../../product/slo-targets.md)
- `ops-ci`
