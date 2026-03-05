---
title: Docker build network policy
audience: operators
type: policy
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - operations
  - docker
related:
  - docs/operations/supply-chain-policies.md
---

# Docker build network policy

Docker build network behavior is governed by `docker/policy.json` `build_network_policy`.

## Allowed

- Explicit package index refresh and install tokens needed for deterministic image assembly.
- Locked cargo build execution.

## Forbidden

- Ad-hoc downloads (`curl`, `wget`, `git clone`)
- Package manager calls outside allowlisted tokens
- Arbitrary installer execution during image build


Enforcement reference: OPS-ROOT-023.
