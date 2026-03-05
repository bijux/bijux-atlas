---
title: Contributor Environment Setup Guide
audience: contributor
type: runbook
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - contributor
---

# Contributor environment setup guide

## Required tools

- Rust toolchain matching repository lockfile
- `cargo` with test and bench support
- `jq` for local JSON artifact inspection

## Setup commands

```bash
cargo fetch
cargo test -p bijux-dev-atlas --no-run
bijux-dev-atlas governance validate --format json
```
