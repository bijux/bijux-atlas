# Vendored Layout Shell Probes

This directory quarantines legacy shell probes that are still referenced by Make lanes.

- These scripts are not part of the `atlasctl` Python package tree.
- New shell checks must not be added under `packages/atlasctl/src/atlasctl/**`.
- Preferred implementation language is Python; shell remains transitional.
