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
  - crates/bijux-atlas/examples/library/getting_started.rs
---

# Library getting started

```rust
use bijux_atlas::model::DatasetId;
use bijux_atlas::query::Region;

fn main() -> Result<(), String> {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38")
        .map_err(|err| err.to_string())?;
    let region = Region::parse("chr1:1000-1250").map_err(|err| err.to_string())?;
    println!("{}", dataset.canonical_string());
    println!("{}", region.canonical_string());
    Ok(())
}
```
