# Security Contracts

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the machine-readable security contract checks and their sources.

## Threat model checks

- `SEC-THREAT-001`: threat model files validate and are complete.
- `SEC-THREAT-002`: every threat has at least one mitigation.
- `SEC-THREAT-003`: every mitigation maps to a concrete control or documented reason.
- `SEC-THREAT-004`: high severity threats map to an executable check or explicit runbook.
- `SEC-AUTH-001`: auth model policy validates and points to the canonical boundary docs.
- `SEC-AUTH-002`: auth policy registry validates and references only known principals, actions, and resources.
- `SEC-AUTH-003`: runtime startup logs declare whether auth is disabled or enforced.
- `SEC-AUTH-004`: the auth-disabled stance keeps a mandatory reverse-proxy boundary in the canonical docs.
- `REL-SEC-AUTH-001`: release evidence includes the auth model and access policy snapshots.

## Compliance checks

- `SEC-COMP-001`: compliance matrix validates.
- `SEC-COMP-002`: every control has at least one evidence pointer.
- `SEC-COMP-003`: evidence pointers resolve to existing files.

## Secret handling checks

- `SEC-RED-001`: redaction policy covers all declared secrets.
- `SEC-RED-002`: the default release evidence directory contains no declared secret matches.
- `SEC-ART-001`: artifact scan passes for the selected directory.
- `REL-AUDIT-001`: release evidence includes the governed audit schema and retention policy.
- `REL-AUDIT-002`: release evidence includes the audit verification report and safe-field inventory.

## Supply-chain checks

- `SEC-DEPS-001`: docs npm dependencies resolve only from the allowlisted registry set.
- `SEC-DEPS-002`: docs Python requirements use only the allowlisted package index set.
- `SEC-IMAGES-001`: governed base images are digest-pinned and recorded in release evidence.
- `SEC-ACTIONS-001`: GitHub Actions refs are SHA-pinned and match the canonical inventory.
- `SEC-SBOM-001`: release evidence includes SBOM coverage that matches prod image digests.

## Source files

- Threat model: `ops/security/threat-model/*.md`, `ops/security/threat-model/*.yaml`
- Compliance: `ops/security/compliance/*.yaml`
- Secret policy: `configs/security/*.json`
- Auth policy: `configs/security/*.yaml`
- Signing policy: `ops/release/signing/policy.yaml`
- Schemas: `configs/contracts/security/*.json`
