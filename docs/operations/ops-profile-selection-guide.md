# Ops Profile Selection Guide

Choose profile by environment and risk tolerance.

- `dev.yaml`: local development and iteration.
- `kind.yaml`: local cluster simulation and CI.
- `prod-minimal.yaml`: single-region conservative production.
- `prod-ha.yaml`: high-availability production rollout.
- `offline.yaml`: air-gapped/offline environments.

Always validate profile contracts before installation.
