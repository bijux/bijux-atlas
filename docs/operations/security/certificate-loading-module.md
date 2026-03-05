---
title: Certificate Loading Module
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Certificate Loading Module

Certificate loading is implemented by `load_certificate_bundle` in core security data protection.

Inputs:

- certificate path
- private key path
- optional CA path

Behavior:

- read PEM files from declared paths
- fail closed on missing or invalid certificate material
