// SPDX-License-Identifier: Apache-2.0

use crate::error::{ClientError, ErrorClass};
use std::thread;
use std::time::Duration;

pub fn run_with_retry<T, F>(attempts: u32, backoff_millis: u64, mut f: F) -> Result<T, ClientError>
where
    F: FnMut() -> Result<T, ClientError>,
{
    let total = attempts.max(1);
    let mut last_error = None;
    for index in 0..total {
        match f() {
            Ok(value) => return Ok(value),
            Err(err) => {
                last_error = Some(err);
                if index + 1 < total {
                    thread::sleep(Duration::from_millis(backoff_millis));
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| ClientError::new(ErrorClass::Client, "retry failure")))
}
