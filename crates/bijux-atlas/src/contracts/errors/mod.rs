// SPDX-License-Identifier: Apache-2.0

mod context;
mod error_codes;
mod model;

pub use context::{ErrorContext, ResultExt};
pub use error_codes::{ErrorCode, ERROR_CODES};
pub use model::{ConfigPathScope, Error, ExitCode, MachineError, Result};
