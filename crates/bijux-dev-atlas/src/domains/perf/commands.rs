// SPDX-License-Identifier: Apache-2.0
//! Performance domain command routes.

use crate::model::CommandRoute;

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "perf",
        "perf",
        "perf",
        "Run performance validation commands",
    )]
}
