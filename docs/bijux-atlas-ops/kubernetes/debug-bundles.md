---
title: Debug Bundles
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Debug Bundles

Atlas preserves Kubernetes diagnostics under one run identity so another
operator can inspect the failure after the cluster has changed. The current
collector is Kind-specific and produces category reports plus captured files;
it is not a generic multi-cluster support archive.

## Capture Model

```mermaid
flowchart LR
    Run[Run identity] --> Logs[kubectl logs]
    Run --> Describe[kubectl describe]
    Run --> Events[kubectl get events]
    Run --> Resources[kubectl get all]
    Logs --> Debug[artifacts/ops/run/debug/namespace]
    Describe --> Debug
    Events --> Debug
    Resources --> Debug
    Debug --> Reports[artifacts/ops/run/reports]
    Reports --> Archive[Deterministic evidence tar]
```

Each collection action writes one captured file and one schema-shaped report:

| Category | Collection | Default file |
| --- | --- | --- |
| `logs` | deployment logs, last 500 lines | `pod-logs.txt` |
| `describe` | Deployment and Service descriptions | `describe.txt` |
| `events` | namespace events sorted by creation time | `events.txt` |
| `resources` | `kubectl get all` in YAML | `resources.yaml` |

Reports are written as
`artifacts/ops/<run-id>/reports/ops-debug-bundle-<category>.json`; captured data
lives below `artifacts/ops/<run-id>/debug/<namespace>/`. The lifecycle evidence
builder can package the run's `reports/` and `debug/` trees into a deterministic
tar with normalized metadata.

## Report Contract

`ops-debug-bundle.schema.json` requires:

- `schema_version: 1`
- `cluster: kind`
- a non-empty `namespace`
- one category from `logs`, `describe`, `events`, or `resources`
- `status` of `ok` or `failed`
- at least one path in `files`

The report records collection status and file membership. It does not record a
source revision, cluster context, capture timestamp, checksums, redaction
result, or incident identifier. Preserve those associations in the surrounding
run evidence; do not infer them from this schema.

## Bind the Capture to a Cluster

Before collection, record the intended kubeconfig source, current context,
cluster identity, namespace, and workload selector. The collector's `kind`
field describes the supported cluster family; it does not prove which Kind
cluster received the commands.

Capture identity and content under one run ID:

```mermaid
sequenceDiagram
    participant Operator
    participant Context as Kubernetes context
    participant Cluster
    participant Run as Run artifact root
    Operator->>Context: resolve current context and namespace
    Context-->>Run: record non-secret cluster identity
    Operator->>Cluster: collect logs, descriptions, events, resources
    Cluster-->>Run: write raw category files
    Operator->>Run: record command status and file membership
    Operator->>Run: checksum, review, and redact distributable copy
```

If the context differs from the incident target, stop. A complete bundle from
the wrong cluster is misleading evidence.

## Separate Observation From Remediation

Capture a pre-change bundle before restarting, rolling back, scaling, or
changing policy. After the mitigation, capture a second bundle under a distinct
capture identity within the same incident record. Never overwrite the first
files with the recovered state.

| Capture | Purpose | Required distinction |
| --- | --- | --- |
| pre-change. | Preserve the failure, workload identity, events, and dependency symptoms. | Timestamp, cluster context, workload revision, and raw-file hashes. |
| action record. | Preserve the exact mutation and its authority. | Command, target, operator, start and end time, and result. |
| post-change. | Demonstrate the resulting identity, readiness, routing, and residual errors. | New capture identity and hashes, linked to but separate from the first capture. |

Comparing these captures can establish what changed during mitigation. It
cannot by itself establish root cause. If collection changes the system—for
example through an expensive query or broad log request—record that effect as
part of the action timeline.

## Capture Before State Disappears

Collect diagnostics before restarting pods, changing a rollout, or deleting a
namespace. Events and logs are perishable. At minimum:

1. capture the failing workload logs
2. describe the Deployment and Service
3. capture namespace events
4. snapshot namespace resources
5. retain the report that triggered collection
6. build the lifecycle evidence archive under the same run identity

The `resources` action uses `kubectl get all`; Kubernetes does not include every
namespaced resource in that alias. Capture ConfigMaps, NetworkPolicies,
Ingresses, PVCs, or custom resources separately when they matter to the
incident.

## Sensitive Data Boundary

The collector writes command output as received. It does not perform automatic
redaction. Before sharing or attaching a bundle:

- inspect environment values, annotations, URLs, headers, and log payloads
- exclude Secrets and credential-bearing custom resources
- preserve enough structure to diagnose the failure without retaining tokens
  or personal data
- record that redaction occurred outside the bundle report, because the current
  schema has no redaction field

An unreviewed bundle is local diagnostic material, not distributable incident
evidence.

Keep the raw capture access-restricted when incident policy permits it. Produce
a separate distributable copy after review, and record its hashes plus the
redaction decision. Never modify the raw files in place and continue using the
original hashes.

## Completeness and Failure Semantics

A category report with `status: ok` means that collection action returned and
the file was written. It does not mean the incident is explained, all four
categories exist, or the evidence archive was built. Conversely, a failed
collection attempt should remain visible; replacing it with an empty successful
report destroys the reason the evidence is incomplete.

## Bundle Acceptance

| Property | Minimum evidence |
| --- | --- |
| attribution | run ID, incident ID, context, cluster, namespace, selector, and capture time |
| integrity | checksum inventory for raw files and report files |
| completeness | requested categories, successes, failures, and supplemental resources |
| confidentiality | reviewer, redaction method, exclusions, and distributable-copy hashes |
| reproducibility | collector version, command arguments, and relevant tool versions |
| custody | capture location, access boundary, archive identity, and retention decision |

An archive is acceptable only for the claim its evidence supports. Four
successful category reports can establish collection completeness. They cannot
establish root cause, safe disclosure, or recovery.

## Authorities

- `ops/schema/k8s/ops-debug-bundle.schema.json`
- `crates/bijux-atlas-ops/src/lifecycle/simulation/debug_collection.rs`
- `crates/bijux-atlas-ops/src/lifecycle/evidence/artifacts.rs`
- `ops/k8s/tests/goldens/k8s-conformance-report.sample.json`
