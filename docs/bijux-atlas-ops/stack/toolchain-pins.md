---
title: Toolchain Pins
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Toolchain and External Input Pins

Atlas governs external inputs at different strengths. Container images and
GitHub Actions have immutable identities. Local operational tools are discovered
and version-checked, but most are not locked to an exact patch release by the
current contracts.

## Pin Strength

| Surface | Current control | Guarantee |
| --- | --- | --- |
| Stack images | Repository, tag, and SHA-256 digest | Immutable image identity when the digest is used at execution time |
| Docker bases | Named builder and runtime image digests | Immutable base input for governed builds |
| GitHub Actions | Human-readable ref plus commit SHA | Workflow action bytes are commit-bound |
| Docker, Syft, Trivy, Helm, Kubeconform, Kind | Allowed major-version policy | Compatibility range, not exact reproducibility |
| Curl, Helm, Kind, kubectl | Required executable plus parsed version | Presence and reportability, not an exact version lock |
| K6 and Kubeconform | Optional executable plus parsed version | Evidence can record use, but absence is permitted by this inventory |

## Resolution Chain

```mermaid
flowchart LR
    P["Pin and version policies"] --> I["Tool and image inventory"]
    I --> M["Generated stack version manifest"]
    M --> R["Runtime or validation run"]
    R --> E["Evidence records resolved versions and digests"]
    E --> D{"Matches declared control?"}
```

The generated stack manifest currently pins the Kind node, MinIO server and
client, Prometheus, OpenTelemetry collector, Redis, and Toxiproxy images. The
stack evolution policy requires pin freeze, version-manifest consistency, and
dependency-graph consistency.

## Reproducibility Rule

A declared pin is useful only when the run proves that it consumed that pin.
Record tool version output, action SHA, image digest observed by the runtime,
generated manifest identity, and source pin inventory. A tag-only component
manifest does not become immutable merely because a digest exists elsewhere.

Do not call a major-version allowance an exact pin. Results produced by Helm
3.14 and Helm 3.16 may both satisfy an `allowed_major: 3` policy while differing
in rendering behavior. When byte-for-byte or semantic reproducibility matters,
the release evidence must bind the exact tool versions used.

## Change Review

For an image or action update, inspect upstream provenance and behavior, change
the authoritative inventory, regenerate dependent manifests, and review the
resulting composition. For a tool compatibility change, review every command
whose output or validation semantics can differ. Preserve old and new versions
with the evidence so drift can be attributed rather than guessed.

## Resolution Failure Modes

| Observation | Trust failure | Required response |
| --- | --- | --- |
| tag resolves to a new digest | mutable reference changed beneath declared intent | reject the run and resolve the approved immutable digest |
| manifest digest and live workload differ | render, admission, mutation, or rollout drift | compare submitted and admitted objects before continuing |
| executable satisfies major policy but output differs | compatible range is too broad for reproducibility claim | bind the exact version and rerun both comparison sides |
| action label and SHA disagree | review label no longer describes executed action | treat commit SHA as execution identity and correct the annotation |
| optional tool is absent | evidence family is unavailable | record the gap and do not emit a passing result for that family |
| tool version cannot be parsed | identity is unknown | fail evidence qualification even if the subprocess otherwise succeeds |

Do not repair a mismatch by rewriting the evidence manifest from the live
environment. First determine whether the governed pin, generated manifest,
rendered intent, or admitted state lost authority.

## Execution Receipt

```mermaid
flowchart LR
    Policy[Allowed version or immutable pin] --> Resolve[Resolved executable or image]
    Resolve --> Invoke[Exact command or workload]
    Invoke --> Output[Output and exit status]
    Output --> Receipt[Policy, identity, input, and output receipt]
```

For local tools, retain executable path, parsed version, package or installation
source where available, command arguments, environment-affecting variables,
and output hash. For images, retain registry, repository, manifest digest,
platform digest, pull policy, and observed workload image ID. For actions,
retain workflow revision, action commit SHA, permissions, and supplied inputs.

The receipt must be produced for the run under review. A generated inventory
showing the intended version cannot prove that a different workstation,
runner, or cluster consumed it.

See [Reproducibility](../release/reproducibility.md) for repeat-build claims and
[Stack Components](stack-components.md) for where pinned images enter a
composition.
