---
title: MSRV policy
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - rust
related:
  - docs/reference/crate-release-policy.md
---

# MSRV policy

- Workspace MSRV is declared in `Cargo.toml` under `workspace.package.rust-version`.
- Publishable crates must inherit workspace MSRV via `package.rust-version.workspace = true`.
- Release validation enforces MSRV alignment through:
  - `bijux dev atlas release validate`

Current MSRV target: `1.84.1`.
