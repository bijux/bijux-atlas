# Load Suites

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

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
- Suite manifest validator: `ops-load-manifest-validate`
- Suite naming convention: `kebab-case`, unique, deterministic.
- Required suite fields: `purpose`, `kind`, `scenario|script`, `thresholds`, `expected_metrics`, `must_pass`.
- Result contract: `ops/load/contracts/result-schema.json`
- `mixed`: baseline mixed traffic distribution.
- `spike-overload-proof`: burst overload behavior.
- `cold-start-p99`: startup latency budget.
- `store-outage-under-spike`: store degradation behavior during spike.
- `pod-churn`: restart churn behavior.
- `cheap-only-survival`: overload cheap-query survival.
- `response-size-abuse`: payload guard enforcement.
- `multi-release`: cross-release query semantics.
- `diff-heavy`: diff endpoint heavy workload profile.
- `mixed-gene-sequence`: combined gene summary and sequence request mix.
- `load-under-rollout`: load while deployment rollout executes.
- `load-under-rollback`: load while deployment rollback executes.
- `sharded-fanout`: shard fanout caps.
- `soak-30m`: long soak with memory growth checks.
- `redis-optional`: redis disabled fallback.
- `ops/load/evaluations/`: non-gating experiment space with strict promotion policy.
- `catalog-federated`: federated registry behavior.

Scenario file names under `ops/load/scenarios/` are internal and not part of the docs surface.
All external references must use suite IDs from `ops/load/suites/suites.json`.

## Budgets

- PR smoke suites must stay within short runtime budget.
- Nightly suites enforce full SLO thresholds from `configs/slo/slo.json`.
- `ops-load-ci` consumes both SLO policy and `suites.json` budgets.

## Failure modes

Scenario drift causes incomplete load coverage.

## How to verify

```bash
$ make ops-load-smoke
$ make ops-load-full
$ make ops-load-ci
$ bijux dev atlas ops load --report text run --suite mixed-80-20
$ bijux dev atlas report ci-summary --latest
```

Expected output: all configured suites produce results and pass policy thresholds.

## See also

- [Load Index](INDEX.md)
- [Load CI Policy](ci-policy.md)
- [SLO Targets](../../product/slo-targets.md)
- `ops-ci`
