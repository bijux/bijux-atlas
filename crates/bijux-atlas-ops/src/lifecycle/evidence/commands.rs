// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;
use std::path::Path;

pub trait EvidenceCommandRunner {
    fn run(&self, binary: &str, args: &[String], cwd: &Path) -> Result<(String, Value), String>;
}

pub fn git_head_sha(
    runner: &impl EvidenceCommandRunner,
    repo_root: &Path,
) -> Result<String, String> {
    let argv = vec!["rev-parse".to_string(), "HEAD".to_string()];
    let (stdout, _) = runner.run("git", &argv, repo_root)?;
    let sha = stdout.trim().to_string();
    if sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(sha)
    } else {
        Err(format!("unexpected git sha output `{sha}`"))
    }
}

pub fn run_security_validate_for_evidence(
    runner: &impl EvidenceCommandRunner,
    repo_root: &Path,
) -> Result<(), String> {
    let argv = vec![
        "run".to_string(),
        "-q".to_string(),
        "-p".to_string(),
        "bijux-atlas-dev".to_string(),
        "--".to_string(),
        "security".to_string(),
        "validate".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    let _ = runner.run("cargo", &argv, repo_root)?;
    let report_path = repo_root.join("artifacts/security/security-github-actions.json");
    if !report_path.exists() {
        return Err(format!(
            "security validate completed without writing {}",
            report_path.display()
        ));
    }
    Ok(())
}

pub fn run_governance_exceptions_validate_for_evidence(
    runner: &impl EvidenceCommandRunner,
    repo_root: &Path,
) -> Result<(), String> {
    let argv = vec![
        "run".to_string(),
        "-q".to_string(),
        "-p".to_string(),
        "bijux-atlas-dev".to_string(),
        "--".to_string(),
        "governance".to_string(),
        "exceptions".to_string(),
        "validate".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    let _ = runner.run("cargo", &argv, repo_root)?;
    let report_path = repo_root.join("artifacts/governance/exceptions-summary.json");
    if !report_path.exists() {
        return Err(format!(
            "governance exceptions validate completed without writing {}",
            report_path.display()
        ));
    }
    Ok(())
}

pub fn run_governance_doctor_for_evidence(
    runner: &impl EvidenceCommandRunner,
    repo_root: &Path,
) -> Result<(), String> {
    for argv in [
        vec![
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-atlas-dev".to_string(),
            "--".to_string(),
            "governance".to_string(),
            "deprecations".to_string(),
            "validate".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
        vec![
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-atlas-dev".to_string(),
            "--".to_string(),
            "governance".to_string(),
            "breaking".to_string(),
            "validate".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
        vec![
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-atlas-dev".to_string(),
            "--".to_string(),
            "governance".to_string(),
            "doctor".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
    ] {
        let _ = runner.run("cargo", &argv, repo_root)?;
    }
    let report_path = repo_root.join("artifacts/governance/governance-doctor.json");
    if !report_path.exists() {
        return Err(format!(
            "governance doctor completed without writing {}",
            report_path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockRunner {
        results: RefCell<VecDeque<Result<(String, Value), String>>>,
    }

    impl EvidenceCommandRunner for MockRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<(String, Value), String> {
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    #[test]
    fn git_head_sha_accepts_owned_hex_output() {
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok((
                "0123456789abcdef0123456789abcdef01234567\n".to_string(),
                serde_json::json!({}),
            ))])),
        };

        let sha = git_head_sha(&runner, Path::new("/repo")).expect("git sha");

        assert_eq!(sha, "0123456789abcdef0123456789abcdef01234567");
    }
}
