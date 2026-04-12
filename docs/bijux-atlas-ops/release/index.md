---
title: Release
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Release

`bijux-atlas-ops/release` explains how Atlas turns a build into a governed
release through manifests, evidence, packets, signing, provenance, drift
review, and rollback readiness.

## Purpose

Use this section to understand which release artifacts are mandatory, how trust
is verified before distribution, and which rollback and reproducibility checks
must be in place before promotion.

## Source of Truth

- `ops/release/release-manifest.json`
- `ops/release/ops-release-manifest.json`
- `ops/release/evidence/`
- `ops/release/packet/packet.json`
- `ops/release/signing/`
- `ops/release/provenance.json`
- `ops/e2e/scenarios/upgrade/`

## Release Control Model

Atlas release control follows this path:

1. a release manifest defines the governed release surfaces
2. evidence collection produces identity, policy, SBOM, package, and report
   artifacts
3. the release packet gathers the minimum portable set for release consumers
4. signing and provenance bind the artifacts to checksums, policy, and source
   identity
5. drift, reproducibility, and rollback readiness decide whether the release is
   safe to distribute

## Pages

- [Backup and Recovery](backup-and-recovery.md)
- [Distribution Channels](distribution-channels.md)
- [Drift Detection](drift-detection.md)
- [Release Evidence](release-evidence.md)
- [Release Packets](release-packets.md)
- [Reproducibility](reproducibility.md)
- [Rollback Drills](rollback-drills.md)
- [Signing and Provenance](signing-and-provenance.md)
- [Upgrades and Rollback](upgrades-and-rollback.md)
- [Version Manifests](version-manifests.md)
