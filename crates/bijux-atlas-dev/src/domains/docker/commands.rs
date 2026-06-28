// SPDX-License-Identifier: Apache-2.0
//! Docker domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "docker",
        "docker",
        "docker",
        "Run docker validation commands",
    )]
}
