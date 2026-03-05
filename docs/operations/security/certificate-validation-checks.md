---
title: Certificate Validation Checks
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Certificate Validation Checks

Certificate validation checks are implemented by `validate_certificate_bundle`.

Checks include:

- certificate PEM marker presence
- private key PEM marker presence
- optional CA PEM marker validation
