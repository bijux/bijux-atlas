// SPDX-License-Identifier: Apache-2.0
//! Governance domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "governance",
        "governance",
        "governance",
        "Inspect governance registries and policy status",
    )]
}
