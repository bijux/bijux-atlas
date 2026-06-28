// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod core {
    pub use bijux_atlas_core::canonical::*;
    pub use bijux_atlas_core::{sha256, sha256_hex, Hash256};
}

pub mod domain {
    pub use crate::core::{sha256, sha256_hex, Hash256};
}

pub mod model {
    pub mod dataset {
        pub use bijux_atlas_model::dataset::*;
    }

    pub mod policy {
        pub use bijux_atlas_model::policy::*;
    }
}

pub mod query {
    pub use bijux_atlas_query::*;
}

pub mod engine;
pub mod version;

pub use engine::*;

pub const CRATE_NAME: &str = "bijux-atlas-ingest";
