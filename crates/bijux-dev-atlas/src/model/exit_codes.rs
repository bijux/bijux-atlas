// SPDX-License-Identifier: Apache-2.0
//! Stable exit code registry for CLI-facing runnable commands.

use crate::engine::RunStatus;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_FAILURE: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_NOT_FOUND: i32 = 3;
pub const EXIT_REQUIRED_FAILURE: i32 = 4;

pub fn exit_code_for_run_status(status: RunStatus) -> i32 {
    match status {
        RunStatus::Pass => EXIT_SUCCESS,
        RunStatus::Skip => EXIT_SUCCESS,
        RunStatus::Fail => EXIT_FAILURE,
        RunStatus::Error => EXIT_REQUIRED_FAILURE,
    }
}
