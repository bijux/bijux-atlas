# Runbooks Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-server`

All runbooks must follow this strict template:

- Symptoms
- Metrics
- Commands
- Expected outputs
- Mitigations
- Rollback
- Postmortem checklist

## Runbooks

- `incident-playbook.md`
- `store-outage.md`
- `dataset-corruption.md`
- `rollback-playbook.md`
- `high-memory.md`
- `k8s-perf-chaos.md`
- `memory-profile-under-load.md`
- `profile-under-load.md`
- `registry-federation.md`
- `traffic-spike.md`
- `pod-churn.md`
- `slo-cheap-burn.md`
- `slo-standard-burn.md`
- `slo-overload-survival.md`
- `slo-registry-refresh-stale.md`
- `slo-store-backend-error-spike.md`

## What

Section overview.

## Why

Rationale for this section.

## Scope

Scope of documents in this section.

## Non-goals

Out-of-scope topics for this section.

## Contracts

Normative links and rules.

## Failure modes

Failure modes if docs drift.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Docs Home](../../index.md)
- [Naming Standard](../../_style/naming-standard.md)
- [Terms Glossary](../../_style/terms-glossary.md)
- `ops-ci`
