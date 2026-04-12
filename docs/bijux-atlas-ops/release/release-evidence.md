---
title: Release Evidence
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Release Evidence

Atlas release decisions depend on explicit evidence under `ops/release/` rather
than on informal operator confidence.

## Purpose

Use this page to understand the minimum release evidence set and what must be
reviewed before a build can be promoted or distributed.

## Source of Truth

- `ops/release/evidence/manifest.json`
- `ops/release/evidence/identity.json`
- `ops/release/evidence/policy.json`
- `ops/release/evidence/sboms/`
- `ops/release/generated/release-metadata.json`

## Required Evidence Set

The release evidence surface currently includes:

- release identity in `identity.json`
- the evidence manifest with package, docs, governance, observability, and
  image references
- policy rules in `policy.json`
- package artifacts and chart packages
- SBOMs for supported profiles
- verification outputs, scan inputs, and human-readable index output

## Human Review Expectations

The evidence should let an operator answer:

- what exact release is being reviewed
- which artifacts belong to it
- whether required package, SBOM, and verification assets are present
- whether governance, observability, and performance evidence stayed attached

## Related Contracts and Assets

- `ops/release/evidence/`
- `ops/release/generated/`
- `ops/release/evidence/manifest.json`
- `ops/release/evidence/identity.json`
- `ops/release/evidence/policy.json`
