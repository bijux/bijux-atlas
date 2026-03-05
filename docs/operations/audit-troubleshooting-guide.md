---
title: Audit Troubleshooting Guide
audience: operator
type: troubleshooting
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - audit
  - troubleshooting
---

# Audit troubleshooting guide

- If `configuration integrity` fails, validate `configs/inventory.json` schema and version.
- If `artifact integrity` fails, validate `release/evidence/manifest.json` exists and is valid JSON.
- If `registry consistency` fails, validate `ops/invariants/registry.json` contains non-empty invariants.
- If `runtime configuration state` fails, validate `ops/k8s/values/offline.yaml` parses as YAML.
- If `ops deployment integrity` fails, validate `ops/k8s/charts/bijux-atlas/Chart.yaml` parses as YAML.
