// SPDX-License-Identifier: Apache-2.0
//! Tutorials domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![
        CommandRoute::new(
            "tutorials",
            "tutorials",
            "tutorials",
            "Run tutorial inventory, workflow, and verification commands",
        ),
        CommandRoute::new(
            "tutorials",
            "tutorials real-data",
            "tutorials",
            "Run governed real-data tutorial catalog, planning, fetch, ingest, and health commands",
        ),
    ]
}
