---
title: API Key Authentication Support
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# API Key Authentication Support

API key support is enabled through `ATLAS_AUTH_MODE=api-key`.

## Control points

- allowed key material contract is validated at startup
- API key flows are exposed through:

```bash
bijux-dev-atlas security authentication api-keys --format json
```

## Related files

- `configs/security/auth-model.yaml`
- `configs/security/runtime-security.yaml`
