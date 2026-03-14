---
title: Library getting started
audience: user
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - examples
related:
  - crates/bijux-atlas/examples/core_contract.rs
---

# Library getting started

```rust
use bijux_atlas_core::DatasetId;

fn main() -> Result<(), bijux_atlas_core::Error> {
    let dataset = DatasetId::new("110/homo_sapiens/GRCh38")?;
    println!("{}", dataset.as_str());
    Ok(())
}
```
