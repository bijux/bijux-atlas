---
title: Makefile Is Wrapper Only
audience: contributor
type: policy
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - make
  - governance
---

# Makefile is wrapper only

`make` is a command-discovery and delegation surface.

Rules:

- wrapper recipes delegate to `bijux-dev-atlas` commands
- no embedded orchestration scripts
- no `control-plane/` or `automation/` call paths
- command behavior lives in Rust control-plane code

Verification:

- `bijux-dev-atlas make wrappers verify`
- `make make-contract-check`
