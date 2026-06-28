// SPDX-License-Identifier: Apache-2.0
//! Ops domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "ops",
        "ops",
        "ops",
        "Run ops runtime and validation commands",
    )]
}
