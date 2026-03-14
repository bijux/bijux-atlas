---
title: Crates you can depend on
audience: user
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - crates
related:
  - docs/reference/crates.md
  - ops/release/crates-v0.1.toml
---

# Crates you can depend on

Use crates listed under `publish.allow` in `ops/release/crates-v0.1.toml`.

- `bijux-atlas`: public Rust crate for the runtime CLI plus embedded API, ingest, store, client, server, query, model, and policy surfaces.

Do not depend on private crates listed in `publish.deny`.
Python consumers should install the `bijux-atlas` SDK package built from `crates/bijux-atlas-python` instead of depending on private Rust crates.
