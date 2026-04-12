---
title: Conformance Suites
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Conformance Suites

Kubernetes conformance is backed by declared suites and manifest-driven test
ownership.

## Purpose

Use this page when you need to understand which Kubernetes checks Atlas runs,
what each suite is meant to prove, and which outputs become release evidence.

## Source of Truth

- `ops/k8s/tests/manifest.json`
- `ops/k8s/tests/suites.json`
- `ops/k8s/tests/ownership.json`
- `ops/k8s/tests/goldens/k8s-conformance-report.sample.json`
- `ops/schema/k8s/conformance-report.schema.json`

## What Is Governed

The Kubernetes conformance program has three governing layers:

- `manifest.json` defines the test inventory, timeout budget, expected failure
  modes, group membership, and quarantine rules
- `suites.json` groups those tests into operator-facing validation lanes such as
  `smoke`, `resilience`, `graceful-degradation`, `api-protection`, and `full`
- `ownership.json` maps each script family to a responsible domain such as
  `chart`, `server`, `store`, `observability`, or `stack`

## Suite Taxonomy

Atlas currently defines these suite classes:

- `smoke` is the fast install and invariant gate for readiness, autoscaling,
  observability wiring, PodDisruptionBudget coverage, and basic sanity
- `resilience` focuses on availability, disruption tolerance, rolling restart
  safety, and autoscaling behavior
- `graceful-degradation` proves survival behaviors such as cached-only mode and
  store outage handling
- `api-protection` checks rate limiting, admission control, and overload
  protection paths
- `full` is the broad integration suite used when release confidence requires
  the whole declared test surface

## Operator Workflow

1. Start from `ops/k8s/install-matrix.json` to identify the suite expected for
   the target profile.
2. Use `ops/k8s/tests/manifest.json` to confirm which scripts cover the change
   surface and what failure modes they are expected to catch.
3. Use `ops/k8s/tests/ownership.json` to route failures to the correct owner.
4. Record the resulting report in the schema-backed conformance format.
5. Treat missing or failing release-gate suites as blockers for promotion.

## How to Read the Evidence

The sample report in
`ops/k8s/tests/goldens/k8s-conformance-report.sample.json` shows the canonical
evidence shape:

- `run_id` identifies the validation attempt
- `suite_id` names the suite that ran
- `status` is the top-level verdict
- `failed_sections` lists broken sections that need review
- `sections.*` records pass or fail results for areas such as `configmap`,
  `networkpolicy`, `pdb`, `probes`, and observability wiring

Pass means the governed contract still holds for the suite scope. Fail means the
surface is unsafe or ambiguous and must be fixed, quarantined with an issue, or
explicitly removed from the release decision.

## Failure Modes

- a required script is missing from the manifest for a changed resource
- a script is quarantined without an issue or beyond the allowed TTL
- a suite passes informally but no schema-backed report is produced
- ownership is unclear, so release-blocking failures bounce across teams
- the changed template surface is not represented in the selected suite

## Evidence Produced

Each conformance run should produce:

- the suite selection and run identifier
- a report that matches `ops/schema/k8s/conformance-report.schema.json`
- section-level failures or missing coverage notes
- ownership mapping for follow-up
- release review notes when a suite is release-blocking

## Related Contracts and Assets

- `ops/k8s/tests/manifest.json`
- `ops/k8s/tests/ownership.json`
- `ops/schema/k8s/conformance-report.schema.json`
- `ops/k8s/tests/suites.json`
- `ops/k8s/tests/goldens/k8s-conformance-report.sample.json`
