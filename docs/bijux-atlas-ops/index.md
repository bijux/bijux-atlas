---
title: bijux-atlas-ops Home
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# bijux-atlas-ops

`bijux-atlas-ops` is the operator handbook for the Atlas control plane. It
explains how runtime stack assets, Kubernetes delivery, observability signals,
load evidence, and release controls fit together when operators need to decide
whether a change is safe to install, promote, or roll back.

## Purpose

Use this handbook to understand the Atlas operating model, choose the right
validation path for a proposed change, and gather the evidence required for
promotion or incident review.

## Source of Truth

- `ops/stack/` defines the runtime dependency shape, local and Kind profiles,
  component manifests, and generated version or dependency views.
- `ops/k8s/` governs Helm values, chart structure, install profiles, rollout
  safety, and Kubernetes conformance evidence.
- `ops/observe/` defines the observability pack, alerting rules, dashboards,
  telemetry drills, and signal contracts.
- `ops/load/` defines scenario identity, thresholds, suites, baselines, k6
  scripts, and generated load summaries.
- `ops/release/` governs release manifests, evidence bundles, packets, signing,
  provenance, and rollback readiness.

## How to Use This Handbook

Read the handbook in the same order operators usually make decisions:

1. Start with [Stack](stack/index.md) to understand component roles, topology,
   profiles, and environment assumptions.
2. Move to [Kubernetes](kubernetes/index.md) for chart values, rendered
   manifests, install paths, rollout controls, and cluster-specific evidence.
3. Use [Observability](observability/index.md) to understand the signal pack
   that validates readiness, health, saturation, and incident triage.
4. Use [Load](load/index.md) to confirm performance, resilience, and regression
   expectations under governed scenarios.
5. Finish with [Release](release/index.md) to package the evidence, verify the
   trust chain, and decide whether a build can be distributed or rolled back.

## What Is Governed

This handbook governs the operator-facing reading of the Atlas control plane:

- supported runtime and dependency layouts
- Kubernetes values, profiles, manifests, and rollout invariants
- observability signals, drills, dashboards, and evidence outputs
- load scenarios, thresholds, baselines, and regression gates
- release manifests, packets, signing outputs, provenance, and recovery

## Operator Workflow

1. Identify the change surface in `ops/stack`, `ops/k8s`, `ops/observe`,
   `ops/load`, or `ops/release`.
2. Read the matching handbook page to confirm the owning contract, validation
   path, and failure modes.
3. Run or inspect the governed checks for that surface.
4. Collect the generated evidence before promotion, rollback approval, or
   incident closure.
5. Cross-check related sections when a decision crosses domain boundaries, such
   as rollout under load or rollback during an observability incident.

## Evidence Produced

The handbook does not generate artifacts by itself, but it points operators to
the evidence that matters for each domain:

- stack indexes, dependency graphs, and version manifests
- rendered manifest inventories and Kubernetes conformance reports
- telemetry indexes, dashboard validation outputs, readiness reports, and drill
  results
- load summaries, baseline comparisons, and threshold regressions
- release manifests, evidence bundles, signed checksums, and verification
  outputs

## What This Handbook Does Not Cover

This handbook is not the place for application feature semantics, dataset
authoring rules, or API consumer integration guidance. Those surfaces live in
their own documentation and contracts. `bijux-atlas-ops` is specifically for
operators who need to reason about deployment safety, runtime behavior,
evidence, and failure handling.

## Sections

- [Stack](stack/index.md)
- [Kubernetes](kubernetes/index.md)
- [Release](release/index.md)
- [Observability](observability/index.md)
- [Load](load/index.md)
