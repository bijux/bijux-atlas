# Release Signing Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: document the governed release signing and provenance contract set.

## Contract Set

- `REL-SIGN-001`: the checksum list exists and covers every required signed artifact.
- `REL-SIGN-002`: checksum entries match the current contents of the referenced files.
- `REL-SIGN-003`: the sign report validates and records every required signing target.
- `REL-SIGN-004`: a freshly generated bundle passes release verification end-to-end.
- `REL-SIGN-005`: the evidence manifest lists the governed signing artifacts explicitly.
- `REL-SIGN-006`: the signing policy points only at existing artifacts.
- `REL-TAR-001`: a freshly rebuilt normalized bundle matches the current bundle except for allowed self-referential generated files.
- `REL-MAN-001`: the generated evidence manifest uses the same schema version declared by the governed manifest schema.
- `REL-PROV-001`: `ops/release/provenance.json` exists and matches the governed provenance schema.

## Validation Surface

- `release sign --evidence ops/release/evidence` generates the checksum ledger, provenance statement, and sign report.
- `release verify --evidence ops/release/evidence/bundle.tar` validates the signing surface and delegates to `ops evidence verify`.
- `ops/release/signing/policy.yaml` is the source of truth for what must be covered.

## Governed Files

- `ops/release/signing/policy.yaml`
- `ops/release/signing/checksums.json`
- `ops/release/signing/release-sign.json`
- `ops/release/signing/release-verify.json`
- `ops/release/provenance.json`
- `configs/contracts/release/*.schema.json`
