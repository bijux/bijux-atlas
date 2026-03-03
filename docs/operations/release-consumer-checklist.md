# Release Consumer Checklist

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Last changed: `2026-03-03`
- Reason to exist: give consumers a short list of checks before deploying a published release.

## Prereqs

- Access to the delivered release bundle and its signing files

## Install

- Verify the delivered bundle path matches `release/evidence/bundle.tar`.
- Verify the checksum list, provenance file, and sign/verify reports are present.

## Verify

- Run `release verify --evidence release/evidence/bundle.tar --format json`.
- Confirm `REL-SIGN-*`, `REL-PROV-001`, `REL-TAR-001`, and `REL-MAN-001` all pass.
- Confirm the chart package and SBOM set are present and match the expected release ID.

## Rollback

- Reject the bundle if any contract fails.
- Request a regenerated and re-verified bundle rather than patching the package in place.
