# Release Trust Root

- Owner: `bijux-atlas-operations`
- Type: `concept`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define what a consumer is expected to trust when verifying a release bundle.

Related ops contracts: `OPS-ROOT-023`, `REL-PROV-001`.

## Trust Root

- `configs/rust/toolchain.json` is the canonical toolchain inventory for governed release generation.
- `ops/release/signing/policy.yaml` is the canonical statement of which artifacts must be covered.
- `ops/release/signing/checksums.json` is the integrity ledger consumers compare against the published files.
- `ops/release/provenance.json` binds the release bundle to one git SHA and one governance version.

## Deterministic Metadata

- Release tarballs are built with stable member ordering.
- Member timestamps are normalized to zero.
- Ownership fields are normalized to empty names and zero numeric IDs.
- Verification commands are local-only and do not depend on remote trust services.
