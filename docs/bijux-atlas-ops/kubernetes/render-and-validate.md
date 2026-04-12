---
title: Render and Validate
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Render and Validate

Atlas treats rendering and validation as first-class operating evidence.

## Purpose

Use this page when turning chart values into deployable manifests and when
proving the rendered output is valid enough to move into conformance, rollout,
or release review.

## Source of Truth

- `makes/entrypoints.mk`
- `makes/k8s.mk`
- `ops/k8s/charts/bijux-atlas/`
- `ops/k8s/generated/`
- `ops/k8s/tests/manifest.json`
- `ops/k8s/tests/goldens/render-kind.summary.json`

## Main Commands

- `make k8s-render`
- `make k8s-validate`
- `make ops-k8s-contracts`

## Validation Pipeline

Atlas expects operators to follow a fixed path:

1. Select a supported profile from `ops/k8s/values/`.
2. Run `make k8s-render` to produce a deterministic manifest report through the
   control plane.
3. Review the render output and generated inventory under
   `artifacts/.../k8s-render/<run-id>/report.json` and `ops/k8s/generated/`.
4. Run `make k8s-validate` to validate the rendered manifests against the
   Kubernetes contract path.
5. Run `make ops-k8s-contracts` when the change needs conformance evidence
   rather than render-only validation.

## Expected Output Artifacts

- `make k8s-render` writes a report to
  `artifacts/.../k8s-render/<run-id>/report.json`
- `make k8s-validate` writes a report to
  `artifacts/.../k8s-validate/<run-id>/report.json`
- `ops/k8s/generated/` updates generated evidence such as
  `inventory-index.json`, `render-artifact-index.json`, and
  `release-snapshot.json`
- `ops/k8s/tests/goldens/render-kind.summary.json` shows the expected summary
  shape for a full render-backed validation run

## How to Interpret Failures

- render failure usually means invalid values, template logic errors, or
  missing source assets
- validation failure means the rendered manifests are not acceptable to the
  control-plane checks or downstream suite expectations
- conformance failure means the rendered output may be structurally valid but is
  operationally unsafe for the selected profile

## Operator Workflow

1. Render first so the desired manifest set is explicit.
2. Validate second so schema, Kubernetes, and contract checks run against the
   exact rendered output.
3. Escalate to conformance when the change affects rollout, security,
   observability, or profile guarantees.
4. Capture the reports and link them into the release or incident record.
