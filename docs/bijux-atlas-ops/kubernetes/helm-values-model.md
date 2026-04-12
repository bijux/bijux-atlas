---
title: Helm Values Model
audience: operators
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Helm Values Model

Atlas treats Helm values as a governed contract surface with explicit schemas,
documentation maps, and install-profile coverage.

## Purpose

Use this page to understand how Atlas separates baseline defaults, install
profiles, documented value ownership, and forbidden high-risk combinations.

## Source of Truth

- `ops/k8s/charts/bijux-atlas/values.yaml`
- `ops/k8s/charts/bijux-atlas/values.schema.json`
- `ops/k8s/values/*.yaml`
- `ops/k8s/values/profiles.json`
- `ops/k8s/values/documentation-map.json`
- `ops/k8s/values-schema-high-risk-policy.json`

## What Is Governed

The values model is intentionally layered:

- `values.yaml` is the baseline contract for the chart
- `values.schema.json` defines valid keys and cross-field rules
- `ops/k8s/values/*.yaml` contains governed install profiles and examples
- `profiles.json` defines the operator intent for supported profiles
- `documentation-map.json` maps top-level keys to the docs surface that owns
  their behavior
- `values-schema-high-risk-policy.json` identifies keys that require extra care
  because they affect security, image provenance, runtime behavior, or storage

## Value Classes

Atlas separates values into four classes:

- baseline defaults: the safe shared starting point in `values.yaml`
- supported overlays: environment or profile values such as `ci.yaml`,
  `kind.yaml`, `offline.yaml`, `perf.yaml`, and `prod.yaml`
- examples and specialist variants: files such as `ingress.yaml`,
  `networkpolicy-custom.yaml`, and `multi-registry.yaml` that describe a
  controlled deployment shape
- forbidden or high-risk toggles: combinations the schema or profile contract
  rejects, such as mutable image tags in performance paths or unsafe debug
  surfaces in production-oriented profiles

## Documentation Ownership

`ops/k8s/values/documentation-map.json` makes the values model traceable. The
map ties top-level keys such as `cache`, `store`, `networkPolicy`, `metrics`,
`rollout`, `probes`, `serviceAccount`, and `rbac` to the docs surface that
should explain them. Some existing entries still point at older documentation
paths, which means this handbook must remain the durable source when those links
are refreshed.

## Operator Workflow

1. Start from `values.yaml`.
2. Select the supported profile in `ops/k8s/values/profiles.json`.
3. Apply the matching values file from `ops/k8s/values/`.
4. Validate the result against `values.schema.json`.
5. Review high-risk keys called out by
   `ops/k8s/values-schema-high-risk-policy.json`.
6. Confirm the change is explained in the owning documentation page before
   promotion.

## How to Validate

- use the schema to reject unknown keys and invalid combinations
- confirm the selected profile still satisfies its required and forbidden
  toggles
- check that high-risk keys such as `image`, `server`, `cache`, `store`,
  `metrics`, `networkPolicy`, `serviceAccount`, and `rbac` have matching review
  evidence
- confirm documentation ownership is still accurate for newly introduced keys

## Failure Modes

- an operator edits a profile file to express a one-off override that should
  have stayed local to validation
- high-risk keys change without rollout, security, or observability review
- examples drift into acting like supported production profiles without being
  listed in `profiles.json`
- documentation claims a key exists, but the schema rejects it or maps it to a
  different operational surface

## Related Contracts and Assets

- `ops/k8s/values/`
- `ops/k8s/values/documentation-map.json`
- `ops/k8s/charts/bijux-atlas/values.schema.json`
- `ops/k8s/charts/bijux-atlas/values.yaml`
- `ops/k8s/values/profiles.json`
- `ops/k8s/values-schema-high-risk-policy.json`
