---
title: API Key Validation
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# API Key Validation

API key validation enforces configured allowed key material and rejects malformed or absent credentials for `api-key` mode.

Use diagnostics command:

```bash
bijux-dev-atlas security authentication diagnostics --format json
```
