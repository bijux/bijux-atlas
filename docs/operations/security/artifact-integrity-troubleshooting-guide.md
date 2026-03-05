---
title: Artifact Integrity Troubleshooting Guide
audience: user
type: guide
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Artifact Integrity Troubleshooting Guide

Symptoms:

- checksum mismatch for dataset artifacts
- manifest integrity verification failure
- tamper detection event bursts

Actions:

1. verify expected checksums in manifest and release evidence
2. re-run strict integrity verification on cached datasets
3. inspect `integrity.violation` and `tamper.detected` counters
4. quarantine affected artifacts and regenerate from trusted source
