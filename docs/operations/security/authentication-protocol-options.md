---
title: Authentication Protocol Options
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Authentication Protocol Options

Atlas supports four protocol modes with clear boundary expectations:

- `api-key`: shared secret header validation for controlled internal callers
- `token`: bearer token verification for signed identity claims
- `oidc`: identity asserted at trusted boundary and forwarded via approved headers
- `mtls`: identity asserted by client certificate and forwarded at trusted boundary

`ATLAS_AUTH_MODE` selects the active mode at runtime.
