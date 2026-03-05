---
title: Installing Ops Bundle
audience: operator
type: how-to
stability: stable
owner: atlas-operations
last_reviewed: 2026-03-05
---

# Installing Ops Bundle

Use this path when you consume a versioned offline ops bundle.

## Prerequisites

- Helm 3.14+
- Kubernetes cluster matching the release compatibility matrix
- Access to the bundle archive and checksums

## Install

1. Build or download bundle:

```bash
cargo run -p bijux-dev-atlas -- ops package --allow-write --allow-subprocess --version 0.1.0 --format json
```

2. Unpack bundle:

```bash
tar -xzf artifacts/release/ops/bundle/v0.1.0/ops-bundle-v0.1.0.tar.gz -C /tmp/atlas-ops
```

3. Install chart values from unpacked bundle:

```bash
helm install atlas /tmp/atlas-ops/charts/bijux-atlas -f /tmp/atlas-ops/values/offline.yaml
```

4. Verify package and readiness:

```bash
cargo run -p bijux-dev-atlas -- release ops validate-package --format json
cargo run -p bijux-dev-atlas -- release ops readiness-summary --format json
```

## Verify

- `http://127.0.0.1:8080/healthz` returns HTTP 200.
- `http://127.0.0.1:8080/readyz` returns HTTP 200.
- readiness summary reports `status: ok`.

## Rollback

```bash
helm rollback atlas 1
```
