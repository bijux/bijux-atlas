# bijux-atlas-model

`bijux-atlas-model` owns the stable Atlas data model boundary: dataset identity
and manifest types, query-facing value objects, diff payloads, and policy
configuration values that need deterministic serde contracts.

Use this crate when you need:

- dataset ids, catalogs, manifests, and shard catalogs
- gene, transcript, seqid, and region model types
- release diff payloads and release gene index records
- policy value objects that should stay outside runtime adapters

Public references:

- Project docs: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-model/latest/bijux_atlas_model/>
- Source repository: <https://github.com/bijux/bijux-atlas>
