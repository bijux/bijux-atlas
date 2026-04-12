---
title: Chart Layout
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Chart Layout

Chart structure lives under `ops/k8s/charts/` and is validated as part of the
Atlas control plane rather than as an isolated Helm artifact.

## Purpose

Use this page to understand where chart logic belongs, which files operators may
edit directly, and which outputs are generated evidence that must never be hand
patched.

## Source of Truth

- `ops/k8s/charts/bijux-atlas/Chart.yaml`
- `ops/k8s/charts/bijux-atlas/values.yaml`
- `ops/k8s/charts/bijux-atlas/values.schema.json`
- `ops/k8s/charts/bijux-atlas/templates/`
- `ops/k8s/generated/`

## What Is Governed

The Atlas chart has a strict ownership split:

- `Chart.yaml` defines chart identity and package metadata
- `values.yaml` holds the baseline operator-facing defaults
- `values.schema.json` defines allowed keys, required relationships, and
  high-risk input rejection
- `templates/` contains the authored manifest logic for deployment, rollout,
  config, policy, service, jobs, and observability wiring
- `ops/k8s/generated/` contains generated inventories and release snapshots
  derived from the authored chart surface

## Layout and Ownership Boundaries

The current chart tree under `ops/k8s/charts/bijux-atlas/` is organized so each
kind of operational concern has a clear home:

- runtime and service wiring live in `templates/deployment.yaml`,
  `templates/rollout.yaml`, `templates/service.yaml`, and
  `templates/configmap.yaml`
- scaling and disruption controls live in `templates/hpa.yaml` and
  `templates/pdb.yaml`
- network and security controls live in `templates/networkpolicy.yaml`,
  `templates/secret.yaml`, and `templates/audit-log-rbac.yaml`
- data warmup and publishing jobs live in `templates/dataset-warmup-job.yaml`
  and `templates/catalog-publish-job.yaml`
- observability integration lives in `templates/prometheusrule.yaml`,
  `templates/prometheusrecordingrule.yaml`, and
  `templates/servicemonitor.yaml`

Keep authored intent in the chart tree. Generated inventories such as
`ops/k8s/generated/render-artifact-index.json` and
`ops/k8s/generated/release-snapshot.json` are evidence of the chart state, not
the place to encode desired behavior.

## How to Validate

1. Edit the authored chart surface only under `ops/k8s/charts/bijux-atlas/`.
2. Validate new values against `values.schema.json`.
3. Render the chart with the intended profile values.
4. Review the generated artifacts in `ops/k8s/generated/` for drift,
   completeness, and release evidence.
5. Run the conformance suites that cover the changed templates.

## Failure Modes

- operators patch generated output instead of the source chart file
- defaults drift from the values schema or profile contracts
- template logic adds a resource without corresponding suite coverage
- a chart change lands without updating the generated inventory or release
  snapshot evidence
- ownership gets blurred between runtime configuration, release metadata, and
  chart rendering

## Evidence Produced

Chart changes should leave behind:

- rendered manifest output
- generated inventory and release snapshot updates in `ops/k8s/generated/`
- conformance suite results for the affected template groups
- any install matrix or rollout evidence required by the target profile

## Related Contracts and Assets

- `ops/k8s/charts/bijux-atlas/Chart.yaml`
- `ops/k8s/charts/bijux-atlas/values.yaml`
- `ops/k8s/charts/bijux-atlas/values.schema.json`
- `ops/k8s/charts/bijux-atlas/templates/`
- `ops/k8s/generated/inventory-index.json`
- `ops/k8s/generated/render-artifact-index.json`
- `ops/k8s/generated/release-snapshot.json`
