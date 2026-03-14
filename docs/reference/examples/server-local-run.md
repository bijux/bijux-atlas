---
title: Server local run example
audience: user
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - examples
related:
  - crates/bijux-atlas/src/bin/bijux-atlas-server.rs
  - configs/examples/runtime/server-minimal.toml
---

# Server local run example

```bash
cargo run -p bijux-atlas --bin bijux-atlas-server -- --config configs/examples/runtime/server-minimal.toml
```

Health checks:

```bash
curl -sf http://127.0.0.1:8080/healthz
curl -sf http://127.0.0.1:8080/readyz
```
