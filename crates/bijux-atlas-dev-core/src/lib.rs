#![forbid(unsafe_code)]

use std::path::PathBuf;

use bijux_atlas_dev_adapters::ProcessAdapter;
use bijux_atlas_dev_model::{CheckResult, CheckStatus, DomainId};

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub repo_root: PathBuf,
    pub domain: Option<DomainId>,
}

pub fn run_checks(_adapter: &dyn ProcessAdapter, request: &RunRequest) -> Result<Vec<CheckResult>, String> {
    let _domain = request.domain.unwrap_or(DomainId::Repo);
    let result = CheckResult {
        id: bijux_atlas_dev_model::CheckId::parse("atlas_dev_bootstrap")?,
        status: CheckStatus::Pass,
        violations: Vec::new(),
        duration_ms: 0,
        evidence: Vec::new(),
    };
    Ok(vec![result])
}
