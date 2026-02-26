// SPDX-License-Identifier: Apache-2.0

use crate::adapters::{DeniedNetwork, FakeWorld, RealFs, RealGit, RealProcessRunner};
use crate::ports::{Clock, Fs, Git, Network, ProcessRunner, SystemClock};

#[derive(Debug, Default)]
pub struct AdaptersBundle {
    pub fs: RealFs,
    pub process: RealProcessRunner,
    pub git: RealGit,
    pub network: DeniedNetwork,
    pub clock: SystemClock,
}

impl AdaptersBundle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filesystem(&self) -> &dyn Fs {
        &self.fs
    }

    pub fn process_runner(&self) -> &dyn ProcessRunner {
        &self.process
    }

    pub fn git(&self) -> &dyn Git {
        &self.git
    }

    pub fn network(&self) -> &dyn Network {
        &self.network
    }

    pub fn clock(&self) -> &dyn Clock {
        &self.clock
    }
}

#[derive(Debug, Default)]
pub struct FixedClock {
    unix_secs: u64,
}

impl FixedClock {
    pub fn new(unix_secs: u64) -> Self {
        Self { unix_secs }
    }
}

impl Clock for FixedClock {
    fn now_unix_secs(&self) -> u64 {
        self.unix_secs
    }
}

#[derive(Debug, Default)]
pub struct TestBundle {
    pub world: FakeWorld,
    pub clock: FixedClock,
}

impl TestBundle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_world(mut self, world: FakeWorld) -> Self {
        self.world = world;
        self
    }

    pub fn with_fixed_time(mut self, unix_secs: u64) -> Self {
        self.clock = FixedClock::new(unix_secs);
        self
    }

    pub fn filesystem(&self) -> &dyn Fs {
        &self.world
    }

    pub fn process_runner(&self) -> &dyn ProcessRunner {
        &self.world
    }

    pub fn clock(&self) -> &dyn Clock {
        &self.clock
    }
}
