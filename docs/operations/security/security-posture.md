# Atlas Security Posture

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Default posture

- Deny-by-default CORS. No cross-origin access unless explicitly allowed by `ATLAS_CORS_ALLOWED_ORIGINS`.
- Strict request limits at ingress:
  - max body bytes (`ATLAS_MAX_BODY_BYTES`)
  - max URI bytes (`ATLAS_MAX_URI_BYTES`)
  - max aggregate header bytes (`ATLAS_MAX_HEADER_BYTES`)
- Normalized request headers for security-sensitive fields (`x-forwarded-for`, `x-api-key`, HMAC headers).

## Authentication and request integrity

- Optional API key support:
  - `ATLAS_ALLOWED_API_KEYS`
  - `ATLAS_REQUIRE_API_KEY`
- Optional HMAC request signing (enterprise mode):
  - `ATLAS_HMAC_SECRET`
  - `ATLAS_HMAC_REQUIRED`
  - `ATLAS_HMAC_MAX_SKEW_SECS`
- Signature payload format:
  - `METHOD + "\n" + PATH_AND_QUERY + "\n" + TIMESTAMP + "\n"`

## Store hardening

- HTTP/S3-like clients disable redirects to reduce SSRF risk.
- URL host validation blocks localhost and literal private IP targets.
- Local filesystem backend enforces canonical parent path checks to block traversal outside store root.

## Abuse resistance

- Per-IP and per-API-key rate limiting.
- Rate-limit bypass prevention through forwarded header normalization.
- Overload shedding and queue depth controls remain active regardless of auth mode.

## Auditability

- Optional machine-readable audit log mode via `ATLAS_ENABLE_AUDIT_LOG=true`.
- Audit records include method, path, status, request id, normalized client IP, and latency.

## See also

- `ops-ci`
