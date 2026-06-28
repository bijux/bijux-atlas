# bijux-atlas-store

`bijux-atlas-store` owns Atlas publish-time storage behavior: artifact layout
keys, manifest locks, immutable publication semantics, and the local or remote
backends used to publish and verify dataset artifacts.

Use this crate when you need:

- publish-time `ArtifactStore` contracts
- deterministic dataset artifact path and key layout
- manifest lock creation and checksum verification
- local filesystem, HTTP readonly, or S3-like artifact backends
- owned storage benches and infrastructure tests

Public references:

- Project docs: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-store/latest/bijux_atlas_store/>
- Source repository: <https://github.com/bijux/bijux-atlas>
