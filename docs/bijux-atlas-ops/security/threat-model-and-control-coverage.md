---
title: Threat Model and Control Coverage
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Threat Model and Control Coverage

The Atlas threat model is a governed relationship among assets, threats,
mitigations, control checks, documentation, and residual risk. It is designed
to make coverage reviewable. It is not a claim that every threat is eliminated
or that every mitigation executed for a particular release.

## Governed Model

```mermaid
flowchart LR
    Asset["asset + sensitivity + owner"] --> Threat["threat + category + severity + likelihood"]
    Threat --> Mitigation["named mitigation"]
    Mitigation --> Check["control check or documented review"]
    Check --> Evidence["fresh result + candidate identity"]
    Evidence --> Residual["residual risk and operating decision"]
```

| Authority | Role |
| --- | --- |
| `ops/security/threat-model/assets.yaml` | names protected runtime, credential, dataset, evidence, report, and audit assets |
| `ops/security/threat-model/threats.yaml` | describes threat category, severity, likelihood, affected component, mitigations, and residual risk |
| `ops/security/threat-model/mitigations.yaml` | maps mitigation identities to checks, operating guides, and review obligations |
| `ops/security/threat-model/classification-taxonomy.yaml` | governs accepted threat categories and classification vocabulary |
| `ops/security/threat-model/threat-registry.yaml` | records the model registry and change authority |
| `ops/security/compliance/controls.yaml` | defines access, audit, privacy, logging, provenance, network, and observability controls |
| `ops/security/compliance/matrix.yaml` | maps controls to expected evidence locations |

Evidence paths in the compliance matrix identify what should support a
control. Their presence does not prove freshness, applicability, or passing
status. Verify the evidence content and bind it to the candidate under review.

## Current Threat Families

| Threat family | Protected concern | Evidence needed beyond registry validity |
| --- | --- | --- |
| runtime spoofing and unauthenticated access | trusted caller boundary and default-deny access | identity mode, route checks, ingress boundary, and audit attribution |
| secret disclosure | credentials in logs, reports, configuration, and evidence | redaction tests, evidence scanning, secret-version handling, and incident detection |
| artifact or evidence tampering | release and review bytes after production | recomputed digests, provenance agreement, consumer verification, and independent trust expectation |
| dependency outage | bounded readiness and serving behavior | failure injection, protected-route behavior, recovery timing, and residual-state proof |
| audit tamper, bypass, or loss | accountable security and operator actions | governed fields, sink continuity, retention, verification, gap detection, and checksum binding |

The registry records residual risk for each threat. Preserve that risk in the
acceptance decision instead of translating a mapped mitigation into “risk
removed.”

## Coverage Is a Chain

```mermaid
flowchart TD
    Declared{"threat and mitigation linked?"} -->|no| Gap["model coverage gap"]
    Declared -->|yes| Implemented{"control implemented at owning boundary?"}
    Implemented -->|no| Gap
    Implemented -->|yes| Exercised{"positive and negative behavior exercised?"}
    Exercised -->|no| Unproven["implemented but unproven claim"]
    Exercised -->|yes| Detected{"audit and detection path observed?"}
    Detected -->|no| Blind["behavior without accountable evidence"]
    Detected -->|yes| Bound{"result bound to release and environment?"}
    Bound -->|no| Observation["unbound observation"]
    Bound -->|yes| Accepted["bounded control claim"]
```

Classify a missing link precisely. A missing mitigation is a model gap. A
missing implementation is a control gap. A missing live exercise is an
assurance gap. A missing audit record is a detection gap. A missing candidate
identity is an evidence-binding gap. These failures have different owners and
must not collapse into a generic security status.

## Evaluate Coverage in Context

A check can support several threats, but its result transfers only when the
enforcement boundary, configuration, environment, and observation window still
match. Counted mappings are inventory evidence, not effectiveness evidence.

| Coverage question | Required context | Common false conclusion |
| --- | --- | --- |
| Is the threat in scope? | asset, exposure, actor, entry point, and impact | a threat is absent because its current registry category differs |
| Is the mitigation present? | implementation revision and effective configuration | a linked check means the control exists in the deployed target |
| Was prevention exercised? | positive and negative case against the owning boundary | a successful allowed path proves denial behavior |
| Was detection exercised? | expected event, sink, correlation, retention, and query result | emitted telemetry remained available and attributable |
| Is the result reusable? | unchanged identities plus a declared validity window | last release's result applies to a changed profile or dependency |
| Is residual risk accepted? | remaining likelihood, impact, compensating controls, owner, and expiry | all mapped checks erase the threat |

```mermaid
flowchart LR
    Mapping["Threat-to-control mapping"] --> Context["Effective target context"]
    Context --> Exercise["Preventive and detective exercise"]
    Exercise --> Finding["Findings and evidence gaps"]
    Finding --> Risk["Residual risk decision"]
    Risk --> Validity["Scope and expiry"]
```

Reopen coverage when an identity used by the result changes or the validity
window expires. A lower-severity environment may reuse implementation evidence,
but it cannot silently transfer an exposure, likelihood, or acceptance decision
to a more privileged target.

## Validate the Model

The focused maintainer command is:

```bash
cargo run --locked -p bijux-atlas-dev -- \
  security threats verify --format json
```

The command validates the governed threat-model contract. Its success does not
execute deployment reachability, runtime authorization, dependency failure, or
consumer release verification. Retain the source revision, model file hashes,
command version, findings, and report status with any review that cites it.

## Change Triggers

Review threat and control coverage when a change introduces or alters:

- a route, authentication mode, principal, role, action, or resource;
- a dataset, cache, store, registry, or external dependency;
- a secret source, transport, retention rule, or redaction boundary;
- a Kubernetes Service, Ingress, NetworkPolicy, service account, RBAC rule,
  volume, or security context;
- an artifact channel, dependency source, image, SBOM, checksum, provenance,
  or verification mechanism; or
- an audit field, sink, rotation, retention, alert, drill, or incident path.

Update the affected asset, threat, mitigation, control, and evidence mappings
together. Adding a control without a threat leaves its rationale unclear;
adding a threat without executable or governed mitigation leaves the risk open.

## Acceptance Record

For every accepted threat boundary, retain the asset and threat IDs,
mitigation and control IDs, implementation revision, executed evidence,
environment and release identity, exceptions, residual risk, decision owner,
and review time. Reopen the decision when any of those identities changes.

Continue with [Security Operations](../kubernetes/security-operations.md) for
deployment enforcement and [Release Evidence](../release/release-evidence.md)
for candidate binding.
