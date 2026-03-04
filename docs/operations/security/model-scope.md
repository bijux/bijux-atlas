# Security Model Scope

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define what the security posture covers and what remains outside the current operating boundary.

## In scope

- Runtime secrets declared in `configs/security/secrets.json`
- Release evidence under `release/evidence/`
- Threat model sources under `security/threat-model/`
- Workflow action pins and dependency sources used by repository-owned automation
- Digest-pinned base images and release SBOM coverage

## Out of scope

- Third-party infrastructure outside repository-governed deployment controls
- End-user identity systems not implemented in this repository
- Managed service provider internals beyond declared interface assumptions
- Consumer-side storage or redistribution after artifacts leave the governed evidence surface

## Threat model entrypoint

- Threat model source of truth: `security/threat-model/README.md`
- [Security Compliance](compliance.md)

## Verify

- Run `bijux-dev-atlas security validate --format json`
- Confirm `SEC-DEPS-*`, `SEC-IMAGES-001`, `SEC-ACTIONS-001`, and `SEC-SBOM-001` pass

## Rollback

- Revert the policy or workflow change that expanded scope without a matching control
- Re-run `bijux-dev-atlas security validate --format json`
