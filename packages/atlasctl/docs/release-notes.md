# Atlasctl Release Notes

This file is the controlled release-notes surface for `atlasctl`.

## Unreleased

- checks control-plane:
  - Added canonical check-id migration map at `configs/policy/check-id-migration.json`.
  - Added `atlasctl checks doctor` and `atlasctl list checks --format tree|json`.
  - Added generated checks registry SSOT artifact at `packages/atlasctl/docs/_meta/checks-registry.txt`.
  - Added registry contract checks for canonical IDs, owners, speed, suite membership, docs/remediation links.

- Added Batch 20 release-readiness gates and docs.
- Added deterministic schema catalog generation (`atlasctl contracts generate --generators catalog`) and schema README sync enforcement.
- Added contract schemas `atlasctl.check-taxonomy.v1` and `atlasctl.suite-manifests.v1`.
- Added `atlasctl contracts validate-self` and made it part of required/release-proof suite gates.

## 0.1.0

- Initial pre-1.0 control-plane release baseline.
