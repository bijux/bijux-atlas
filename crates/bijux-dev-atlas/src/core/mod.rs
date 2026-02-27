// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
//! `core` contains the pure control-plane engine, registries, and checks.
//!
//! Boundary: core may depend on `model`, `policies`, and `ports`; direct host effects belong in
//! `adapters` implementations.

#[cfg(test)]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[cfg(test)]
use crate::model::Visibility;
use crate::model::{
    ArtifactsRoot, CheckId, CheckResult, CheckSpec, CheckStatus, DomainId, Effect, RunId,
    RunReport, RunSummary, Severity, SuiteId, Tag, Violation,
};
use std::borrow::Cow;

mod check_runner;
pub mod checks;
pub mod logging;
pub mod ops_registry;
#[path = "inventory.rs"]
pub mod ops_inventory;
mod registry;
#[path = "report.rs"]
mod report_rendering;
pub use crate::ports::{Capabilities, Fs, ProcessRunner};

pub fn load_dev_policy_set(
    repo_root: &Path,
) -> Result<crate::policies::DevAtlasPolicySet, Cow<'static, str>> {
    crate::policies::DevAtlasPolicySet::load(repo_root).map_err(|err| Cow::Owned(err.to_string()))
}

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub repo_root: PathBuf,
    pub domain: Option<DomainId>,
    pub capabilities: Capabilities,
    pub artifacts_root: Option<PathBuf>,
    pub run_id: Option<RunId>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Selectors {
    pub id_glob: Option<String>,
    pub domain: Option<DomainId>,
    pub tag: Option<Tag>,
    pub suite: Option<SuiteId>,
    pub include_internal: bool,
    pub include_slow: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub fail_fast: bool,
    pub max_failures: Option<usize>,
}

#[derive(Debug)]
pub enum CheckError {
    Failed(String),
}

pub type CheckFn = fn(&CheckContext<'_>) -> Result<Vec<Violation>, CheckError>;

pub trait EffectsBoundary {
    fn filesystem(&self) -> &dyn Fs;
    fn process_runner(&self) -> &dyn ProcessRunner;
}

pub struct AdapterSet<'a> {
    pub fs: &'a dyn Fs,
    pub process: &'a dyn ProcessRunner,
}

impl EffectsBoundary for AdapterSet<'_> {
    fn filesystem(&self) -> &dyn Fs {
        self.fs
    }

    fn process_runner(&self) -> &dyn ProcessRunner {
        self.process
    }
}

pub struct CheckContext<'a> {
    pub repo_root: &'a Path,
    pub artifacts_root: PathBuf,
    pub run_id: RunId,
    pub adapters: AdapterSet<'a>,
    pub registry: &'a Registry,
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub repo_root: PathBuf,
    pub artifacts_root: ArtifactsRoot,
    pub run_id: RunId,
    pub capabilities: Capabilities,
}

impl RuntimeContext {
    pub fn from_run_request(request: &RunRequest) -> Result<Self, String> {
        let artifacts_root = match &request.artifacts_root {
            Some(path) => ArtifactsRoot::parse(&path.display().to_string())?,
            None => ArtifactsRoot::default_for_repo(&request.repo_root),
        };
        let run_id = request
            .run_id
            .clone()
            .unwrap_or_else(|| RunId::from_seed("registry_run"));
        Ok(Self {
            repo_root: request.repo_root.clone(),
            artifacts_root,
            run_id,
            capabilities: request.capabilities,
        })
    }

    pub fn check_artifacts_run_root(&self) -> PathBuf {
        self.artifacts_root.to_path_buf().join(self.run_id.as_str())
    }
}

#[cfg(test)]
pub(crate) use check_runner::evidence_path_has_timestamp;
pub use check_runner::{run_checks, Check, CheckRunner};
pub use registry::{
    expand_suite, explain_output, list_output, load_registry, registry_doctor, select_checks,
    validate_registry, Registry, RegistryDoctorReport, SuiteSpec, DEFAULT_REGISTRY_PATH,
};
#[cfg(test)]
pub(crate) use registry::{
    parse_effect_for_test as parse_effect,
    registry_ordering_errors_for_test as registry_ordering_errors,
};
pub use report_rendering::{
    exit_code_for_report, render_ci_summary_line, render_json, render_jsonl, render_text_summary,
    render_text_with_durations,
};

#[cfg(test)]
#[path = "../../tests/support/core_engine_tests.rs"]
mod lib_tests;
