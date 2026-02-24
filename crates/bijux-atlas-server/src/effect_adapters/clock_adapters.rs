// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)] // ATLAS-EXC-0001

use std::time::{Duration, Instant};

pub(crate) fn now() -> Instant {
    Instant::now()
}

pub(crate) fn add_ms(base: Instant, ms: u64) -> Instant {
    base + Duration::from_millis(ms)
}
