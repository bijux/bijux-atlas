---
title: Debug Bundles
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Debug Bundles

Debug bundles are governed artifacts for cluster and runtime diagnosis, not
ad hoc tarballs.

## Purpose

Use a debug bundle when an install, rollout, readiness incident, or conformance
failure needs a durable evidence package that another operator can inspect
without reconnecting to the cluster.

## Source of Truth

- `ops/schema/k8s/ops-debug-bundle.schema.json`
- `ops/k8s/tests/goldens/k8s-conformance-report.sample.json`
- `ops/report/generated/`

## What Is Governed

The schema-backed bundle record captures:

- `cluster`, which is currently expected to be `kind`
- `namespace`, so evidence stays tied to a concrete surface
- `category`, which must be one of `logs`, `describe`, `events`, or
  `resources`
- `status`, which records whether the capture succeeded
- `files`, which lists the collected bundle members

## Bundle Contents

A complete debug bundle should include the files needed to explain the failure
without guesswork:

- pod and workload descriptions for the failing namespace
- recent events around scheduling, probes, and rollout transitions
- selected logs for the affected Atlas workload and supporting jobs
- rendered resources or manifest snapshots that show the intended state
- the matching conformance, validation, or rollout report when one exists

## Redaction and Naming Rules

- exclude secrets and credentials rather than masking them late
- avoid raw environment dumps that may expose sensitive values
- name files so the category, namespace, and workload are obvious to a later
  reader
- keep one bundle per incident or validation run so evidence does not blur
  across unrelated failures

## When to Capture a Bundle

Capture a debug bundle whenever:

- `k8s-validate` fails and the rendered output does not explain the failure by
  itself
- a rollout stalls, flaps readiness, or triggers rollback review
- a conformance suite reports failing sections that need owner handoff
- an incident needs logs, describe output, and event history preserved before
  the cluster changes again

## How to Validate

1. Confirm the bundle metadata matches
   `ops/schema/k8s/ops-debug-bundle.schema.json`.
2. Check that every file listed in `files` actually exists in the captured
   evidence set.
3. Verify no sensitive content escaped redaction rules.
4. Link the bundle to the relevant rollout, conformance, or incident record.

## Failure Modes

- bundle capture happens too late and the useful events have rotated away
- operators collect logs only and miss the rendered or describe evidence
- sensitive content enters the bundle and blocks distribution
- bundle filenames are generic, making later incident review ambiguous

## Evidence Produced

The debug bundle itself is the evidence artifact. It should accompany:

- the failing validation or incident identifier
- the namespace and cluster used during capture
- the status of the capture attempt
- references to the rollout, conformance, or report outputs that explain why it
  was collected

## Related Contracts and Assets

- `ops/schema/k8s/ops-debug-bundle.schema.json`
- `ops/report/generated/`
- `ops/k8s/tests/goldens/k8s-conformance-report.sample.json`
