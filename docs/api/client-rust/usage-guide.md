# Rust Client Usage Guide

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`

## Install

Add dependency:

```toml
bijux-atlas-client = { path = "crates/bijux-atlas-client" }
```

## Minimal Query

```rust
use bijux_atlas_client::{AtlasClient, ClientConfig, DatasetQuery};

let client = AtlasClient::new(ClientConfig::default())?;
let query = DatasetQuery::new("110", "homo_sapiens", "GRCh38");
let page = client.dataset_query(&query, None)?;
```

## Examples

- `simple-query`
- `dataset-scan`
- `filtered-query`
- `streaming-results`
- `pagination`
- `client-cli`
