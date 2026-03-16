---
title: Feature Flags
audience: maintainer
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Feature Flags

This page summarizes the notable Cargo feature flags in the current `bijux-atlas` crate.

## Feature Flag Groups

```mermaid
flowchart LR
    Features[Cargo features] --> Runtime[Runtime capabilities]
    Features --> Bench[Benchmark enablement]
    Features --> Allocator[Allocator choice]
```

This feature-group diagram shows why Cargo features belong in reference rather than runtime
configuration docs. They are build-time capability switches, not live server knobs.

## Current Features

```mermaid
flowchart TD
    FeatureSet[Feature set] --> Default[default]
    FeatureSet --> Local[backend-local]
    FeatureSet --> S3[backend-s3]
    FeatureSet --> Bench[bench-ingest-throughput]
    FeatureSet --> Jemalloc[jemalloc]
```

This current-feature map gives a fast inventory of the notable compile-time switches in the crate.

## Feature Summary

- `default`
- `serde`
- `backend-local`
- `backend-s3`
- `bench-ingest-throughput`
- `jemalloc`

## Reading Guidance

Treat feature flags as build-time capability switches. Do not confuse them with runtime configuration flags.

## Purpose

This page is the lookup reference for feature flags. Use it when you need the current checked-in surface quickly and without extra narrative.

## Stability

This page is a checked-in reference surface. Keep it synchronized with the repository state and generated evidence it summarizes.
