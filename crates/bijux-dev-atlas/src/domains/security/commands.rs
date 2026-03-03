// SPDX-License-Identifier: Apache-2.0
//! Security domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "security",
        "security",
        "security",
        "Run security validation commands",
    )]
}
