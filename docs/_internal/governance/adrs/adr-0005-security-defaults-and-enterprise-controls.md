# ADR-0005: Security Defaults And Optional Enterprise Controls

Status: Accepted

Context:
Atlas must be safe by default and support stronger controls in regulated environments.

Decision:
- Deny-by-default CORS and strict request size limits.
- Optional API key and HMAC request signing controls.
- SSRF-safe store clients and local path traversal checks.

Consequences:
- Safer defaults for internet-exposed deployments.
- More config surface that operators must manage intentionally.
