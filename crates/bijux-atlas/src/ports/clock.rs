// SPDX-License-Identifier: Apache-2.0

use std::time::{Duration, SystemTime};

use crate::errors::Result;

pub trait ClockPort {
    fn now(&self) -> Result<SystemTime>;
    fn sleep(&self, duration: Duration) -> Result<()>;
}
