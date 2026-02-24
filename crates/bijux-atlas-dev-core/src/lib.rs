#![forbid(unsafe_code)]

use std::path::PathBuf;

use bijux_atlas_dev_adapters::ProcessAdapter;
use bijux_atlas_dev_model::{CheckDomain, CheckResult};

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub repo_root: PathBuf,
    pub domain: Option<CheckDomain>,
}

pub fn run_checks(_adapter: &dyn ProcessAdapter, request: &RunRequest) -> Result<Vec<CheckResult>, String> {
    let domain = request.domain.unwrap_or(CheckDomain::Repo);
    let result = CheckResult {
        check_id: "atlas_dev_bootstrap".to_string(),
        domain,
        passed: true,
        violations: Vec::new(),
    };
    Ok(vec![result])
}
