# bijux-atlas

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![bijux-atlas](https://img.shields.io/crates/v/bijux-atlas?label=bijux--atlas&logo=rust)](https://crates.io/crates/bijux-atlas)
[![ghcr-bijux--atlas](https://img.shields.io/badge/ghcr-bijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas)
[![rust-docs](https://img.shields.io/badge/rust--docs-bijux--atlas-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas/latest/bijux_atlas/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas` is the published compatibility facade for the Atlas Rust library
family. It preserves the compact `bijux_atlas::...` namespace while ownership
and implementation remain in focused crates.

It is not the Atlas executable and it is not a monolithic implementation. The
`bijux-atlas` command comes from `bijux-atlas-cli`; the service process comes
from `bijux-atlas-server`.

## Add the Facade

```toml
[dependencies]
bijux-atlas = "0.2"
```

The facade supports compatibility imports such as:

```rust
use bijux_atlas::query::Region;
use bijux_atlas::domain::ingest::*;
```

The directly owned equivalents remain available from their leaf crates:

```rust
use bijux_atlas_query::Region;
```

Choose the facade when a stable, compact import path is more important than a
minimal dependency graph. Choose a leaf crate when the application needs only
one domain or when precise ownership is preferable.

## Exported Surface

| Facade path | Canonical owner |
| --- | --- |
| `bijux_atlas::api::*` | `bijux-atlas-api` |
| `bijux_atlas::query::*` | `bijux-atlas-query` |
| `bijux_atlas::domain::ingest::*` | `bijux-atlas-ingest` |
| `bijux_atlas::domain::{canonical, cluster, policy, security, time}` | `bijux-atlas-runtime` |
| `bijux_atlas::{adapters, app, contracts, packaged, runtime}` | `bijux-atlas-runtime` |

The facade also re-exports runtime identity, environment, hashing, version, and
no-randomness-policy symbols. Rust API documentation is the authoritative list
for a published version.

## Ownership Boundaries

- `bijux-atlas-runtime` composes the product library and owns runtime contracts.
- `bijux-atlas-api` owns API types and OpenAPI generation.
- `bijux-atlas-query` owns query types and planning.
- `bijux-atlas-ingest` owns normalization and immutable artifact construction.
- `bijux-atlas-cli` owns end-user command dispatch.
- `bijux-atlas-server` owns HTTP delivery.
- `bijux-atlas-ops` owns operational contracts and repository surface models.

These boundaries prevent a convenience import from obscuring the component
that defines behavior. Compatibility additions belong here only when they
preserve an intentional public path; new domain behavior belongs in its owner
crate.

## Compatibility Policy

The facade and its leaf dependencies share the Atlas workspace version. A
facade re-export is part of the public Rust surface and follows the repository's
compatibility policy. Internal module layout in an owner crate is not made
stable merely because the facade depends on that crate.

Applications that require the smallest semver and compile-time surface should
depend on owner crates directly. Applications preserving existing
`bijux_atlas::...` paths should use this facade.

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API: <https://docs.rs/bijux-atlas/latest/bijux_atlas/>
- source: <https://github.com/bijux/bijux-atlas>
