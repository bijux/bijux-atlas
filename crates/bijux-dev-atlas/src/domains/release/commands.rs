// SPDX-License-Identifier: Apache-2.0
//! Release domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "release",
        "release",
        "release",
        "Run release verification commands",
    )]
}
