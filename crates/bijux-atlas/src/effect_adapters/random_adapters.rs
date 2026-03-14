// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)] // ATLAS-EXC-0001

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_u64() -> u64 {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
