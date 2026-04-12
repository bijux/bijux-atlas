---
title: Kubernetes
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Kubernetes

`bijux-atlas-ops/kubernetes` explains the Atlas Kubernetes operating model. The
section treats Kubernetes as a governed contract system rather than a loose set
of Helm templates.

## Purpose

Use this section when you need to understand how Atlas turns Helm values and
profile intent into rendered manifests, cluster installs, rollout decisions,
security reviews, and conformance evidence.

## Source of Truth

- `ops/k8s/charts/bijux-atlas/` defines the chart structure, baseline values,
  and values schema.
- `ops/k8s/values/` defines install profiles, overlays, examples, and
  documentation mappings.
- `ops/k8s/install-matrix.json` defines supported install and upgrade paths.
- `ops/k8s/rollout-safety-contract.json` defines promotion and rollback
  invariants.
- `ops/k8s/profile-security-contract.json` and
  `ops/k8s/admin-endpoints-exceptions.json` define security expectations and
  explicit exceptions.
- `ops/k8s/tests/manifest.json`, `ops/k8s/tests/suites.json`, and
  `ops/k8s/tests/goldens/` define conformance execution and evidence.

## What Is Governed

The Kubernetes slice governs:

- chart ownership boundaries and generated output discipline
- valid values, profile intent, and high-risk value restrictions
- supported install, upgrade, rollback, offline, and air-gapped paths
- rollout readiness, drain, safety gates, and rollback triggers
- network policy expectations, security posture, and approved exceptions
- rendered manifest validation and conformance evidence

## Kubernetes Control Model

Atlas uses a layered control model:

1. Operators start with the chart baseline in
   `ops/k8s/charts/bijux-atlas/values.yaml`.
2. A governed profile or overlay from `ops/k8s/values/*.yaml` selects the
   intended environment shape.
3. `values.schema.json` and related policies reject unsupported or risky input
   combinations before render.
4. Rendered manifests become the deployable contract and are checked for schema,
   Kubernetes validity, and suite coverage.
5. Rollout safety and security review decide whether the rendered output may be
   promoted.
6. Conformance reports and generated inventories become release evidence.

## How to Validate

Use the pages in this section in the same order an operator would validate a
cluster change:

1. Confirm value ownership in [Helm Values Model](helm-values-model.md).
2. Check supported deployment paths in [Install Matrix](install-matrix.md).
3. Follow the render pipeline in [Render and Validate](render-and-validate.md).
4. Review promotion gates in [Rollout Safety](rollout-safety.md).
5. Review security posture in [Security Operations](security-operations.md) and
   [Admin Endpoints Exceptions](admin-endpoints-exceptions.md).
6. Inspect evidence expectations in [Conformance Suites](conformance-suites.md)
   and [Debug Bundles](debug-bundles.md).

## Pages

- [Admin Endpoints Exceptions](admin-endpoints-exceptions.md)
- [Chart Layout](chart-layout.md)
- [Conformance Suites](conformance-suites.md)
- [Debug Bundles](debug-bundles.md)
- [Helm Values Model](helm-values-model.md)
- [Install Matrix](install-matrix.md)
- [Render and Validate](render-and-validate.md)
- [Rollout Safety](rollout-safety.md)
- [Runtime Configuration](runtime-configuration.md)
- [Security Operations](security-operations.md)
