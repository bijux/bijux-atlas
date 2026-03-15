// SPDX-License-Identifier: Apache-2.0

use crate::contracts::errors::Result;

pub trait MetricsPort {
    fn increment_counter(&self, name: &str, value: u64) -> Result<()>;
}

pub trait TracingPort {
    fn record_span(&self, name: &str) -> Result<()>;
}
