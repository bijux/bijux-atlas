# Ops Acceptance Checklist

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## What

Canonical checklist for local/CI ops acceptance.

## Why

Maps acceptance criteria to one make target each, so verification is deterministic.

## Scope

Local stack bring-up, deploy, warm/smoke, k8s gates, load smoke, observability validation.

## Non-goals

Does not replace deep-dive runbooks.

## Contracts

- Ref-grade local required gate: `ops-ref-grade-local` via `make ops-ref-grade-local`
- Ref-grade PR variant: `ops-ref-grade-pr` via `make ops-ref-grade-pr`
- Ref-grade nightly variant: `ops-ref-grade-nightly` via `make ops-ref-grade-nightly`
- Stack up: `ops-up` via `make ops-up`
- Deploy app: `ops-deploy` via `make ops-deploy`
- Warm cache: `ops-warm` via `make ops-warm`
- Smoke API: `ops-smoke` via `make ops-smoke`
- K8s tests: `ops-k8s-tests` via `make ops-k8s-tests`
- Load smoke: `ops-load-smoke` via `make ops-load-smoke`
- Observability validate: `ops-observability-validate` via `make ops-observability-validate`
- Full acceptance: `ops-full` via `make ops-full`

## Failure modes

- Missing target mapping causes ambiguous acceptance.
- Target drift causes false-positive ops readiness.

## How to verify

```bash
$ make ops-ref-grade-local
```

Expected output: every stage exits 0 and writes artifacts under `artifacts/ops/`.

## See also

- [Operations Index](INDEX.md)
- [Canonical Workflows](canonical-workflows.md)
- [Terms Glossary](../_style/terms-glossary.md)
