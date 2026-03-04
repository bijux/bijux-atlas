---
title: Identity Access And Data Models
audience: contributor
type: concept
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-04
tags:
  - architecture
  - security
---

# Identity Access And Data Models

## Identity Model

Atlas principal model includes:

- user
- service-account
- operator
- ci

Principal attributes include stable identifier, source, and declared role bindings.

## Access Control Model

Access evaluation combines:

- authenticated principal identity
- requested action identifier
- target resource scope
- policy and role bindings

Default behavior for missing policy is deny.

## Data Protection Model

Data protection model defines:

- sensitive data classes and handling constraints
- transport protection requirements
- artifact integrity and tamper detection expectations
- audit visibility without secret disclosure
