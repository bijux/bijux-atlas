// SPDX-License-Identifier: Apache-2.0
//! Configs domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "configs",
        "configs",
        "configs",
        "Run configs validation and explanation commands",
    )]
}
