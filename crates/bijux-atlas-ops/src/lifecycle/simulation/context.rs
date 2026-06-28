// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::access_guard::ensure_simulation_cluster_context;
use serde_json::Value;
use std::path::Path;

pub trait SimulationCommandRunner {
    fn run(&self, binary: &str, args: &[String], cwd: &Path) -> Result<(String, Value), String>;
}

pub fn ensure_owned_simulation_context(
    runner: &impl SimulationCommandRunner,
    force: bool,
) -> Result<(), String> {
    struct Adapter<'a, T>(&'a T);

    impl<T: SimulationCommandRunner> crate::kubernetes::execution::KubernetesCommandRunner
        for Adapter<'_, T>
    {
        fn run(
            &self,
            binary: &str,
            args: &[String],
            cwd: &Path,
        ) -> Result<crate::kubernetes::execution::SubprocessCapture, String> {
            let (stdout, event) = self.0.run(binary, args, cwd)?;
            Ok(crate::kubernetes::execution::SubprocessCapture { stdout, event })
        }
    }

    ensure_simulation_cluster_context(&Adapter(runner), force)
}
