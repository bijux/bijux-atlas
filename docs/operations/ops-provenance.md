---
title: Ops provenance
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Ops provenance

This document defines the minimum provenance chain for published ops artifacts.

## Required references

- `release/ops-release-manifest.json`: published chart reference and digest binding.
- `release/ops-release-bundle-manifest.json`: chart version and workspace version binding.
- `release/ops-chart-digest.json`: local package hash used before publish.
- `ops/report/generated/ops-artifact-lineage.json`: chart digest, image digest, and evidence schema linkage.

## Verification intent

- A consumer can match the chart digest in `release/ops-release-manifest.json` with `release/ops-chart-digest.json`.
- A consumer can validate chart and workspace version linkage in `release/ops-release-bundle-manifest.json`.
- A reviewer can trace image and chart lineage through `ops/report/generated/ops-artifact-lineage.json`.
