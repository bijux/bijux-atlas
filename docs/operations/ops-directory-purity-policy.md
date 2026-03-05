---
title: Ops Directory Purity Policy
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - operations
  - governance
---

# Ops Directory Purity Policy

`ops/` is reserved for operational artifacts and configuration assets.

## Required rules

- `ops/` must not contain `*.py` files.
- `ops/` must not contain `*.sh` files.
- `ops/` files must use approved artifact extensions.

Approved extensions:

- `json`
- `jsonl`
- `yaml`
- `yml`
- `toml`
- `md`
- `js`
- `txt`
- `lock`
- `sqlite`
- `prom`
- `gz`
- `fa`
- `fai`
- `gff3`
- `tpl`
- `mmd`
- `env`
- `.gitkeep` (filename allowlist)

## Enforcement

Repository contracts in `crates/bijux-dev-atlas/tests/repo_automation_doctrine_contracts.rs` enforce these constraints on every test run.


Enforcement reference: OPS-ROOT-023.
