---
title: Threat Model Troubleshooting
audience: user
type: guide
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Threat Model Troubleshooting

Common failures:

- threat references unknown mitigation ID
- threat references unknown asset ID
- threat ID missing from registry index
- category/severity/likelihood not in taxonomy

Primary command:

```bash
bijux-dev-atlas security threats verify --format json
```
