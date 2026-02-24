mod context;
mod model;

pub use context::{ErrorContext, ResultExt};
pub use model::{ConfigPathScope, Error, ErrorCode, ExitCode, MachineError, Result, ERROR_CODES};
