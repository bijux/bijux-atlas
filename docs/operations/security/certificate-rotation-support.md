---
title: Certificate Rotation Support
audience: user
type: guide
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Certificate Rotation Support

Certificate rotation is supported by `CertificateRotationState` in core.

## Rotation flow

1. stage next certificate fingerprint
2. validate staged bundle
3. activate staged fingerprint after validation
4. retain audit trail of activation event
