---
title: Distribution Channels
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Distribution Channels

Atlas separates package publication, OCI bundle publication, GitHub release
assets, documentation deployment, chart distribution, and offline delivery.
These channels carry different media and are not interchangeable merely because
they share a release tag.

```mermaid
flowchart TB
    Source[Tagged source revision] --> Build[Channel-specific build]
    Build --> Crates[crates.io packages]
    Build --> OCI[GHCR OCI release bundles]
    Build --> GitHub[GitHub release assets]
    Build --> Docs[GitHub Pages site]
    Build --> Chart[Helm OCI chart]
    Build --> Offline[Offline operations bundle]
    Crates --> Verify[Cross-channel identity review]
    OCI --> Verify
    GitHub --> Verify
    Docs --> Verify
    Chart --> Verify
    Offline --> Verify
```

## Channel Map

| Channel | Declared authority | Publication mechanism | Consumer verifies |
| --- | --- | --- | --- |
| Rust crates | `crates-release.toml` | `release-crates.yml` and crates.io | package name, version, checksum, dependency closure |
| GHCR bundles | release workflow matrices | `release-ghcr.yml` packages build artifacts with ORAS | OCI reference, tag, digest, artifact media type |
| GitHub release | release plan and notes | `release-github.yml` uploads release artifacts | tag, source revision, assets, checksums, notes |
| documentation | MkDocs configuration | `deploy-docs.yml` builds and deploys Pages | site version, source revision, canonical URLs |
| Helm chart | `ops-release.toml` | control-plane package and push commands | chart version, OCI digest, values schema, install evidence |
| offline bundle | `ops-release.toml` and bundle manifest | control-plane bundle commands | manifest, included assets, checksums, offline install evidence |

`release-ghcr.yml` publishes tarred release artifacts as generic OCI artifacts
using ORAS. That is distinct from proving that a runnable multi-architecture
container image satisfying `images-release.toml` was built. Check the artifact
media type and contents instead of inferring “container image” from GHCR alone.

## Declared and Automated Surfaces

The checked-in GitHub workflows directly automate crates.io publication, GHCR
OCI bundle publication, GitHub release publication, and documentation deploy.
Helm packaging, Helm OCI push, and offline-bundle assembly exist as
`bijux-atlas-dev` release commands, but no dedicated checked-in workflow invokes
those operations end to end.

The former documentation reference to `.github/workflows/docker-publish.yml`
was invalid; that file does not exist. Runtime-image policy remains declared in
`ops/release/images-release.toml`, while the active GHCR workflow publishes
release bundles according to caller-provided matrices.

## Cross-Channel Coherence

A release is coherent only when all promoted channels resolve to the same
release identity:

- workspace and package versions match the intended tag
- OCI and chart digests identify the artifacts named in provenance
- GitHub assets match the checksum and signing records
- deployed documentation describes the released, not merely checked-out,
  surface
- the offline bundle contains the exact chart, profiles, images, tools, and
  evidence it declares

A successful channel publish proves only that channel. Crates.io success does
not prove chart availability; a GitHub release does not prove an offline
install; a GHCR tag does not prove its media type or digest matches another
manifest.

## Channel Completion States

| State | Evidence | Promotion consequence |
| --- | --- | --- |
| planned | channel, artifact, version, and expected identity are declared | no publication claim |
| uploaded | producer reports a successful upload | retrieval and remote identity still unproven |
| resolved | consumer can retrieve the immutable reference and recompute its identity | channel delivery established |
| cross-checked | version, digest, provenance, and packet records agree with required sibling channels | channel can participate in release completion |
| failed | upload, retrieval, identity, or policy check fails | release remains partial and promotion is held |

Use immutable digests or registry checksums for completion. A mutable tag, web
page, or package search result is useful for discovery but cannot by itself
bind the received bytes to the release packet.

## Partial Publication Reconciliation

```mermaid
flowchart TD
    Attempt[Publish required channels] --> Inventory[Record each remote result]
    Inventory --> Complete{All required identities resolve?}
    Complete -->|yes| Cross[Cross-channel coherence check]
    Complete -->|no| Hold[Hold promotion]
    Hold --> Inspect[Inspect immutable successes and failed channel]
    Inspect --> Decide{Same candidate can be resumed safely?}
    Decide -->|yes| Resume[Publish only missing safe operations]
    Decide -->|no| Withdraw[Record withdrawal or superseding release]
    Cross --> Consumer[Consumer verification]
```

Do not delete the record of a partial attempt. It explains why some registries
contain a version that was never promoted. Before resuming, prove that local
candidate bytes still match already published immutable references and that
the channel permits the remaining operation without overwriting history.

## Current Manifest Limit

The workspace version is `0.2.2`, while several checked-in release and
operations bundle manifests still identify `0.2.0`. Treat those files as stale
evidence fixtures until they are regenerated and verified together. They must
not be cited as the distribution manifest for the current workspace version.

## Promotion Checklist

1. Resolve the tag, source revision, and channel-specific manifest.
2. Build the channel artifact from that revision.
3. Record the immutable digest or registry checksum.
4. Attach signing, provenance, SBOM, and channel-specific validation.
5. Verify the consumer retrieval path, not only the producer upload path.
6. Compare version and digest identities across every channel being promoted.
7. Retain a verifier result that fails on missing or mismatched artifacts.

Also record the required-channel set. Optional channels may legitimately lag,
but that policy must be declared before publication; reclassifying a failed
required channel as optional after the fact weakens the release contract.

## Authorities

- `.github/workflows/release-crates.yml`
- `.github/workflows/release-ghcr.yml`
- `.github/workflows/release-github.yml`
- `.github/workflows/deploy-docs.yml`
- `ops/release/crates-release.toml`
- `ops/release/images-release.toml`
- `ops/release/ops-release.toml`
- `ops/release/ops-release-bundle-manifest.json`
