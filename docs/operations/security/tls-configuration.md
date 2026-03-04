# TLS Configuration

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: configure runtime TLS and certificate material.

## Runtime Inputs

- `ATLAS_REQUIRE_HTTPS`
- `transport.tls_required`
- `transport.min_tls_version`
- certificate, private key, and optional CA files

## Validation

- certificate and key files must exist and be non-empty
- certificate files must contain PEM certificate markers
- key files must contain PEM private key markers

## Rotation

- stage next certificate fingerprint
- promote next fingerprint to active
- verify traffic and error rates before removing retired material
