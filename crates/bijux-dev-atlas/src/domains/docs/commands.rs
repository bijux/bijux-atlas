// SPDX-License-Identifier: Apache-2.0
//! Docs domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "docs",
        "docs",
        "docs",
        "Run docs validation and generation commands",
    )]
}
