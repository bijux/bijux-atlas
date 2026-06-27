// SPDX-License-Identifier: Apache-2.0

mod context;
mod model;

pub use bijux_atlas_core::{ErrorCode, ERROR_CODES};
pub use context::{ErrorContext, ResultExt};
pub use model::{ConfigPathScope, Error, ExitCode, MachineError, Result};
