# Datasets Configs

This directory is the source of truth for governed dataset manifests and pinning policy.

- `manifest.yaml` defines the approved dataset catalog and its provenance metadata.
- `pinned-policy.yaml` defines which deployment profiles must pin datasets and which IDs are allowed.

Update these files through the control-plane validation path:

- `bijux dev atlas datasets validate`
- `bijux dev atlas ingest dry-run --dataset <id>`
