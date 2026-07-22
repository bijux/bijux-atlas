---
title: Backup and Recovery
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Backup and Recovery

Atlas recovery begins with immutable dataset artifacts, catalog and manifest
identity, release metadata, configuration, and evidence. Cache state is
disposable. Runtime rollback and dataset-pointer rollback are useful recovery
tools, but neither replaces a durable backup and tested restore.

## Recovery Domains

```mermaid
flowchart TD
    Incident{"What was lost or corrupted?"}
    Incident -->|runtime only| Runtime["Restore supported release"]
    Incident -->|dataset selection| Pointer["Publish previous verified pointer"]
    Incident -->|artifact or catalog| Store["Restore durable store and catalog"]
    Incident -->|evidence or control state| Control["Restore manifests, policy, and provenance"]
    Runtime --> Verify["Verify identity, correctness, readiness, and traffic"]
    Pointer --> Verify
    Store --> Verify
    Control --> Verify
```

## Protection Inventory

| State | Protection requirement | Recovery proof |
| --- | --- | --- |
| Dataset artifacts and `manifest.lock` | Immutable, replicated, checksum-verified copy | Artifacts validate and governed queries match |
| Catalog and dataset index | Versioned copy or deterministic rebuild inputs | Expected releases and datasets are discoverable |
| Release packet and provenance | Retained outside the failed environment | Consumer verification passes after restore |
| Runtime configuration and secrets | Versioned policy plus protected secret backup | Render and security contracts pass |
| Cache | No authoritative backup required | Rebuild from verified store without result drift |

Define recovery-point and recovery-time objectives for each durable class. Test
the same encryption keys, credentials, network boundary, storage class, and
catalog scale used by the target environment. A file copy is not recovered
service until identity, integrity, readiness, and representative queries pass.

## Recovery Objectives

| Objective | Governing question | Required measurement |
| --- | --- | --- |
| recovery point | How much committed state may be lost? | newest restorable identity and gap from incident time |
| recovery time | How long may the service remain unavailable or degraded? | detection-to-restored-service timeline |
| integrity | Can restored bytes be trusted? | manifest, checksum, signature, and catalog verification |
| completeness | Are all required durable classes present? | artifact, catalog, release, configuration, and secret inventory |
| usability | Can the intended workload resume safely? | readiness plus representative correctness and load checks |

Objectives apply per durable class. A current catalog backup paired with stale
artifact bytes is not a coherent recovery point. Restoring bytes inside the
time objective does not pass recovery if identity or query correctness fails.

## Coherent Recovery Set

```mermaid
flowchart TD
    RP[Named recovery point] --> Artifacts[Dataset artifacts and manifest]
    RP --> Catalog[Catalog and dataset index]
    RP --> Release[Runtime and release metadata]
    RP --> Config[Configuration and policy]
    RP --> Access[Keys, credentials, and access path]
    Artifacts --> Bind{Identities and hashes agree?}
    Catalog --> Bind
    Release --> Bind
    Config --> Bind
    Access --> Bind
    Bind -->|no| Reject[Reject incoherent restore]
    Bind -->|yes| Exercise[Exercise representative workload]
    Exercise --> Verdict[Independent recovery verdict]
```

The backup unit is the coherent recovery set, not an individual archive.
Artifact bytes without their manifest cannot establish identity. Encrypted
backups without recoverable keys cannot establish availability. A restored
catalog that selects a different artifact generation cannot establish the
declared recovery point.

Preserve the dependency order for each environment. Recover the access path
needed to read protected backups, then authoritative artifacts and catalog,
then policy and runtime configuration, and only then rebuild disposable caches.
Do not allow a rebuilt cache to become evidence for an unverified store.

## Recover Access Without Collapsing Trust

Recovery credentials, encryption keys, trust roots, and verifier policy are
dependencies of the restore, not ordinary members of the same untrusted
archive. If every access and verification input is protected by the system it
must unlock, the backup may be intact but operationally unrecoverable.

| Trust input | Independent recovery requirement | Acceptance evidence |
| --- | --- | --- |
| encryption key | recoverable key generation under separate custody and quorum or break-glass policy | key generation opens the named backup without exposing key material |
| storage credential | least-privilege restore identity with audited issuance and expiry | only the selected recovery set is readable or writable as intended |
| release expectation | expected outer digest or release identity outside the restored packet | restored members match the independently supplied identity |
| trust and withdrawal policy | current verifier roots, policy, revocations, and time source | verifier records the policy identity and current authorization result |
| target authority | named approver for restore, traffic transfer, and writer activation | decision record binds recovery point, target, and accepted evidence |

Use separate identities for reading the backup, materializing the isolated
restore, verifying it, and activating service. Record each grant and revoke
temporary access after the boundary it owns completes. A break-glass action
must leave an audit record and cannot substitute for restored least-privilege
operation.

## Restore Validation Sequence

```mermaid
flowchart LR
    Select[Select named recovery point] --> Isolate[Isolate failed state]
    Isolate --> Restore[Restore durable classes]
    Restore --> Integrity[Verify bytes, manifests, and catalog]
    Integrity --> Configure[Restore policy, configuration, and secrets]
    Configure --> Admit[Start without normal traffic]
    Admit --> Query[Run representative correctness checks]
    Query --> Load[Observe bounded traffic and dependencies]
    Load --> Resume[Resume service or reject recovery]
```

Keep the failed state and restore evidence isolated until the verdict is
recorded. A failed restore is diagnostic evidence and must not overwrite the
last known recoverable point.

## Recovery State and Serving Authority

Recovery has several distinct states. Treating them as synonyms creates the
most dangerous restore failure: a system that is reachable but serves an
unproven combination of data, catalog, runtime, and policy.

```mermaid
stateDiagram-v2
    [*] --> BackupPresent
    BackupPresent --> Restorable: inventory, access, and integrity verify
    Restorable --> Restored: coherent set materialized in isolation
    Restored --> ServingCandidate: runtime starts and identity checks pass
    ServingCandidate --> Qualified: correctness and bounded traffic pass
    ServingCandidate --> Rejected: identity, correctness, or capacity fails
    Rejected --> Restorable: preserve evidence and select a trusted point
    Qualified --> Authoritative: governed failover transfers traffic and writes
```

| State | What has been established | What remains prohibited |
| --- | --- | --- |
| backup present | an archive or snapshot is discoverable | assuming it is readable, complete, or correctly keyed |
| restorable | the coherent set can be read and its inventory and integrity agree | modifying the failed environment or accepting service traffic |
| restored | artifacts, catalog, release metadata, configuration, and access state exist in isolation | advertising the target as production-ready |
| serving candidate | the runtime starts and returns the expected release and dataset identities | normal traffic, writes, and promotion |
| qualified | representative correctness, dependency, and bounded-load checks pass | authority transfer without a recorded decision |
| authoritative | one governed endpoint, catalog pointer, and writer path have been transferred | reactivating the retired writer or stale endpoint |

Measure the recovery-point interval from the last accepted mutation or
publication included in the selected recovery set to the first mutation known
to be excluded. Measure recovery time from the declared incident or recovery
start to the recorded authority transfer, with detection, isolation, restore,
qualification, and failover timestamps retained separately. Configured
objectives, archive timestamps, and a successful process start are inputs to
those measurements; none proves the observed result.

If only part of the set restores, stop before serving. Do not combine a newer
catalog with older artifacts, recover encrypted bytes without the matching key
generation, or reuse a cache populated from the rejected target. Preserve the
partial result, return to a known restorable point, and repeat the complete
identity and qualification chain.

## Prevent Split Authority

Restore into an isolated target while the failed environment remains frozen.
Only one catalog pointer, runtime release, and administrative path may be
authoritative when service resumes. A successful query against the restore
target does not by itself transfer authority.

| Transition | Required control |
| --- | --- |
| incident to isolation. | Stop publication and administrative mutation, capture the active identities, and fence the failed writer path. |
| isolation to validation. | Expose the restore only to verification traffic and reject unverified catalog or artifact combinations. |
| validation to failover. | Record the accepted recovery point, switch one governed authority, and observe representative traffic. |
| failover to normal service. | Confirm no stale endpoint, pointer, credential, or job can mutate the retired state. |
| later failback. | Treat the original environment as a new recovery target; reconcile state and repeat validation before switching. |

Inventory every catalog publication or administrative mutation accepted after
the selected recovery point. Classify it as retained, replayed, superseded, or
lost. This reconciliation is the observed data-loss interval; a backup
timestamp alone cannot establish it.

## Exercise and Acceptance Authority

A recovery exercise records the selected point, backup age, restore start and
end, recovered identities, validation results, data-loss interval, and operator
verdict. Measure recovery point and recovery time from observed timestamps;
archive timestamps or workflow duration alone are insufficient.

When practical, a person other than the restore executor should confirm the
manifest and catalog binding, representative query results, and traffic-ready
state. Independence protects against repeating the same assumption that caused
or concealed the incident. If independent verification is unavailable, record
that limitation with the acceptance decision.

Test more than the successful path. An unreadable object, missing key, partial
catalog, stale manifest, and unavailable rollback target should fail closed
without damaging the last verified recovery point. A restore procedure that
has only been read or rendered is not a tested recovery capability.

## Current Repository Boundary

The repository defines a dataset pointer rollback policy with a maximum depth
of three and retains release manifests, packets, and provenance contracts. It
does not contain an operational backup configuration, backup schedule, storage
retention policy, restore runner, or completed recovery result.

The checked-in MinIO stack manifest is a single-replica end-to-end fixture with
development credentials and no persistent volume. It cannot support a durable
backup claim. The release packet is also stale and the current evidence bundle
fails verification.

Accordingly, Atlas currently documents reconstruction inputs and rollback
policy, not a proven disaster-recovery system. An environment must supply and
exercise the missing backup, retention, restore, credential, and off-site
controls before claiming recoverability.

## Recovery Acceptance

Preserve the incident scope, selected recovery point, backup identity and age,
restore timestamps, artifact and catalog checksums, release verification,
configuration and secret restoration, readiness, representative query results,
load behavior, and residual data-loss assessment. Test failure paths, including
an unreadable backup and an unavailable previous release.

Do not resume writes or promotion when artifact integrity is uncertain. Isolate
the affected release, restore from a verified point, and retain the failed
evidence for investigation.

See [Cache and Store Operations](../stack/cache-and-store-operations.md) for
state authority and [Release Evidence](release-evidence.md) for current packet
limitations.
