# Ops Release

This document defines what shipping the ops product means for Bijux Atlas.

## Distribution Model

Ops is shipped through two public artifacts:

- OCI Helm chart: `oci://ghcr.io/<owner>/charts/bijux-atlas`
- Versioned offline bundle tarball: `ops-bundle-v<version>.tar.gz`

The release source of truth is [`release/ops-v0.1.toml`](../release/ops-v0.1.toml).

## Required Build Command

Use only `bijux-dev-atlas` commands:

```bash
cargo run -p bijux-dev-atlas -- ops package --allow-write --allow-subprocess --version 0.1.0 --format json
```

This command packages the chart and builds the versioned ops bundle.

## Required Verification Commands

```bash
cargo run -p bijux-dev-atlas -- release ops validate-package --format json
cargo run -p bijux-dev-atlas -- release ops readiness-summary --format json
```

## Shipping Criteria

A release is shippable only when all are true:

- chart package exists in the configured package output directory,
- bundle tarball exists in the configured bundle output directory,
- bundle checksums and manifest artifacts are generated,
- readiness summary is `ok`.

## Reproducibility Requirement

Ops bundle outputs must be reproducible from repository state:

- deterministic input list is declared in `release/ops-v0.1.toml`,
- contracts verify the reproducibility policy and deterministic bundle settings,
- CI tag jobs run package + bundle build with artifact upload.
