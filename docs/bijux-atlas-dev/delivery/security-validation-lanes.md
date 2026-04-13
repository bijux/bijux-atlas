---
title: Security Validation Lanes
audience: maintainers
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Security Validation Lanes

Security workflows are split into supply chain, threat model, and data
protection validation lanes.

## Security Lane Model

```mermaid
flowchart TD
    Security[Security validation] --> Vulnerability[Vulnerability scanning]
    Security --> SupplyChain[Supply-chain checks]
    Security --> Policy[Policy and security rules]
    Security --> Publish[Publish-time verification]

    Vulnerability --> Evidence[Security evidence set]
    SupplyChain --> Evidence
    Policy --> Evidence
    Publish --> Evidence

    Evidence --> Block{Release-blocking issue?}
    Block -- Yes --> Stop[Stop promotion]
    Block -- No --> Proceed[Proceed with recorded evidence]
```

This model helps maintainers read the security workflows as a coordinated set of
gates instead of a random list of CI files.

## Workflow Anchors

- [`.github/workflows/security-supply-chain-validation.yml`](/Users/bijan/bijux/bijux-atlas/.github/workflows/security-supply-chain-validation.yml:1)
- [`.github/workflows/security-threat-model-validation.yml`](/Users/bijan/bijux/bijux-atlas/.github/workflows/security-threat-model-validation.yml:1)
- [`.github/workflows/security-data-protection-validation.yml`](/Users/bijan/bijux/bijux-atlas/.github/workflows/security-data-protection-validation.yml:1)

## Main Takeaway

Security validation lanes are release-shaping evidence paths. They exist so
maintainers can separate dependency and supply-chain review, threat-model
governance, and publish-time security confidence instead of folding them into
one vague notion of "security checked."
