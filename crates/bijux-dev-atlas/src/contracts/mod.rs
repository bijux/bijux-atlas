// SPDX-License-Identifier: Apache-2.0
//! Contracts runner engine.
//!
//! This module provides a domain-agnostic contracts runner with deterministic ordering,
//! filterable execution, pretty and JSON output, and explicit effect gating.

use std::cmp::Ordering;
use std::fmt;
use std::path::{Path, PathBuf};

pub mod docker;

pub trait ContractRegistry {
    fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String>;
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContractId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TestId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestKind {
    Pure,
    Subprocess,
    Network,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Static,
    Effect,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static => write!(f, "static"),
            Self::Effect => write!(f, "effect"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub contract_id: String,
    pub test_id: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub message: String,
    pub evidence: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TestResult {
    Pass,
    Fail(Vec<Violation>),
    Skip(String),
    Error(String),
}

pub struct TestCase {
    pub id: TestId,
    pub title: &'static str,
    pub kind: TestKind,
    pub run: fn(&RunContext) -> TestResult,
}

pub struct Contract {
    pub id: ContractId,
    pub title: &'static str,
    pub tests: Vec<TestCase>,
}

pub struct RunContext {
    pub repo_root: PathBuf,
    pub artifacts_root: Option<PathBuf>,
    pub mode: Mode,
    pub allow_subprocess: bool,
    pub allow_network: bool,
    pub skip_missing_tools: bool,
    pub timeout_seconds: u64,
}

pub struct RunOptions {
    pub mode: Mode,
    pub allow_subprocess: bool,
    pub allow_network: bool,
    pub skip_missing_tools: bool,
    pub timeout_seconds: u64,
    pub fail_fast: bool,
    pub contract_filter: Option<String>,
    pub test_filter: Option<String>,
    pub list_only: bool,
    pub artifacts_root: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseStatus {
    Pass,
    Fail,
    Skip,
    Error,
}

impl CaseStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
            Self::Error => "ERROR",
        }
    }

    fn as_colored(self) -> &'static str {
        match self {
            Self::Pass => "\u{1b}[32mPASS\u{1b}[0m",
            Self::Fail => "\u{1b}[31mFAIL\u{1b}[0m",
            Self::Skip => "\u{1b}[33mSKIP\u{1b}[0m",
            Self::Error => "\u{1b}[31mERROR\u{1b}[0m",
        }
    }
}

pub struct CaseReport {
    pub contract_id: String,
    pub contract_title: String,
    pub test_id: String,
    pub test_title: String,
    pub kind: TestKind,
    pub status: CaseStatus,
    pub violations: Vec<Violation>,
    pub note: Option<String>,
}

pub struct ContractSummary {
    pub id: String,
    pub title: String,
    pub status: CaseStatus,
}

pub struct RunReport {
    pub domain: String,
    pub mode: Mode,
    pub contracts: Vec<ContractSummary>,
    pub cases: Vec<CaseReport>,
}

impl RunReport {
    pub fn total_contracts(&self) -> usize {
        self.contracts.len()
    }

    pub fn total_tests(&self) -> usize {
        self.cases.len()
    }

    pub fn pass_count(&self) -> usize {
        self.cases
            .iter()
            .filter(|c| c.status == CaseStatus::Pass)
            .count()
    }

    pub fn fail_count(&self) -> usize {
        self.cases
            .iter()
            .filter(|c| c.status == CaseStatus::Fail)
            .count()
    }

    pub fn skip_count(&self) -> usize {
        self.cases
            .iter()
            .filter(|c| c.status == CaseStatus::Skip)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.cases
            .iter()
            .filter(|c| c.status == CaseStatus::Error)
            .count()
    }

    pub fn exit_code(&self) -> i32 {
        if self.error_count() > 0 {
            1
        } else if self.fail_count() > 0 {
            2
        } else {
            0
        }
    }
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let mut regex = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            _ => regex.push_str(&regex::escape(&ch.to_string())),
        }
    }
    regex.push('$');
    regex::Regex::new(&regex)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

fn matches_filter(filter: &Option<String>, value: &str) -> bool {
    filter
        .as_deref()
        .map(|f| wildcard_match(f, value))
        .unwrap_or(true)
}

fn case_status_from_result(result: &TestResult) -> CaseStatus {
    match result {
        TestResult::Pass => CaseStatus::Pass,
        TestResult::Fail(_) => CaseStatus::Fail,
        TestResult::Skip(_) => CaseStatus::Skip,
        TestResult::Error(_) => CaseStatus::Error,
    }
}

fn worst_status(current: CaseStatus, next: CaseStatus) -> CaseStatus {
    fn rank(s: CaseStatus) -> u8 {
        match s {
            CaseStatus::Pass => 0,
            CaseStatus::Skip => 1,
            CaseStatus::Fail => 2,
            CaseStatus::Error => 3,
        }
    }
    match rank(current).cmp(&rank(next)) {
        Ordering::Less => next,
        _ => current,
    }
}

pub fn run(
    domain: &str,
    contracts_fn: fn(&Path) -> Result<Vec<Contract>, String>,
    repo_root: &Path,
    options: &RunOptions,
) -> Result<RunReport, String> {
    let mut contracts = contracts_fn(repo_root)?;
    contracts.sort_by_key(|c| c.id.0.clone());

    let ctx = RunContext {
        repo_root: repo_root.to_path_buf(),
        artifacts_root: options.artifacts_root.clone(),
        mode: options.mode,
        allow_subprocess: options.allow_subprocess,
        allow_network: options.allow_network,
        skip_missing_tools: options.skip_missing_tools,
        timeout_seconds: options.timeout_seconds,
    };

    let mut contract_rows = Vec::new();
    let mut case_rows = Vec::new();

    for contract in contracts {
        if !matches_filter(&options.contract_filter, &contract.id.0) {
            continue;
        }
        let mut cases = contract.tests;
        cases.sort_by_key(|t| t.id.0.clone());
        let mut contract_status = CaseStatus::Pass;
        let mut has_case = false;
        for case in cases {
            if !matches_filter(&options.test_filter, &case.id.0) {
                continue;
            }
            has_case = true;
            let result = if options.list_only {
                TestResult::Skip("list-only".to_string())
            } else {
                match (options.mode, case.kind) {
                    (Mode::Static, TestKind::Subprocess | TestKind::Network) => {
                        TestResult::Skip("effect-only test".to_string())
                    }
                    (Mode::Effect, TestKind::Subprocess) if !options.allow_subprocess => {
                        TestResult::Error("requires --allow-subprocess".to_string())
                    }
                    (Mode::Effect, TestKind::Network) if !options.allow_network => {
                        TestResult::Error("requires --allow-network".to_string())
                    }
                    _ => match std::panic::catch_unwind(|| (case.run)(&ctx)) {
                        Ok(v) => v,
                        Err(_) => TestResult::Error("test panicked".to_string()),
                    },
                }
            };
            let status = case_status_from_result(&result);
            contract_status = worst_status(contract_status, status);
            let (violations, note) = match result {
                TestResult::Pass => (Vec::new(), None),
                TestResult::Fail(rows) => (rows, None),
                TestResult::Skip(reason) => (Vec::new(), Some(reason)),
                TestResult::Error(err) => (Vec::new(), Some(err)),
            };
            case_rows.push(CaseReport {
                contract_id: contract.id.0.clone(),
                contract_title: contract.title.to_string(),
                test_id: case.id.0,
                test_title: case.title.to_string(),
                kind: case.kind,
                status,
                violations,
                note,
            });
            if options.fail_fast && matches!(status, CaseStatus::Fail | CaseStatus::Error) {
                break;
            }
        }
        if has_case {
            contract_rows.push(ContractSummary {
                id: contract.id.0,
                title: contract.title.to_string(),
                status: contract_status,
            });
        }
        if options.fail_fast && contract_status == CaseStatus::Error {
            break;
        }
    }

    let report = RunReport {
        domain: domain.to_string(),
        mode: options.mode,
        contracts: contract_rows,
        cases: case_rows,
    };

    if let Some(root) = &options.artifacts_root {
        let out_dir = root.join("contracts");
        std::fs::create_dir_all(&out_dir)
            .map_err(|e| format!("create contracts artifact dir failed: {e}"))?;
        let json_path = out_dir.join(format!("{domain}.json"));
        std::fs::write(
            &json_path,
            serde_json::to_string_pretty(&to_json(&report))
                .map_err(|e| format!("encode contracts report failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", json_path.display()))?;
    }

    Ok(report)
}

pub fn to_pretty(report: &RunReport) -> String {
    fn dotted(label: &str, status: &str) -> String {
        const WIDTH: usize = 72;
        let left = if label.len() >= WIDTH {
            label.to_string()
        } else {
            format!("{label} {}", ".".repeat(WIDTH - label.len()))
        };
        format!("{left} {status}")
    }

    let mut out = String::new();
    out.push_str(&format!(
        "Contracts: {} (mode={})\n",
        report.domain, report.mode
    ));
    for contract in &report.contracts {
        out.push_str(&format!(
            "{}\n",
            dotted(
                &format!("{} {}", contract.id, contract.title),
                contract.status.as_colored()
            )
        ));
        for case in report
            .cases
            .iter()
            .filter(|c| c.contract_id == contract.id)
        {
            out.push_str(&format!(
                "  {}\n",
                dotted(&case.test_id, case.status.as_colored())
            ));
            for violation in &case.violations {
                let location = match (&violation.file, violation.line) {
                    (Some(file), Some(line)) => format!("{file}:{line}"),
                    (Some(file), None) => file.clone(),
                    _ => "unknown-location".to_string(),
                };
                out.push_str(&format!("    - {}: {}\n", location, violation.message));
                if let Some(evidence) = &violation.evidence {
                    out.push_str(&format!("      evidence: {}\n", evidence.trim()));
                }
            }
            if let Some(note) = &case.note {
                out.push_str(&format!("    - note: {note}\n"));
            }
        }
    }
    out.push_str(&format!(
        "Summary: {} contracts, {} tests: {} pass, {} fail, {} skip, {} error\n",
        report.total_contracts(),
        report.total_tests(),
        report.pass_count(),
        report.fail_count(),
        report.skip_count(),
        report.error_count()
    ));
    out
}

pub fn to_json(report: &RunReport) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "domain": report.domain,
        "mode": report.mode.to_string(),
        "summary": {
            "contracts": report.total_contracts(),
            "tests": report.total_tests(),
            "pass": report.pass_count(),
            "fail": report.fail_count(),
            "skip": report.skip_count(),
            "error": report.error_count(),
            "exit_code": report.exit_code()
        },
        "contracts": report.contracts.iter().map(|c| serde_json::json!({
            "id": c.id,
            "title": c.title,
            "status": c.status.as_str()
        })).collect::<Vec<_>>(),
        "tests": report.cases.iter().map(|t| serde_json::json!({
            "contract_id": t.contract_id,
            "contract_title": t.contract_title,
            "test_id": t.test_id,
            "test_title": t.test_title,
            "kind": format!("{:?}", t.kind).to_ascii_lowercase(),
            "status": t.status.as_str(),
            "note": t.note,
            "violations": t.violations.iter().map(|v| serde_json::json!({
                "contract_id": v.contract_id,
                "test_id": v.test_id,
                "file": v.file,
                "line": v.line,
                "message": v.message,
                "evidence": v.evidence
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
        fn pass_case(_: &RunContext) -> TestResult {
            TestResult::Pass
        }
        Ok(vec![Contract {
            id: ContractId("DOCKER-001".to_string()),
            title: "sample",
            tests: vec![TestCase {
                id: TestId("docker.sample.pass".to_string()),
                title: "sample pass",
                kind: TestKind::Pure,
                run: pass_case,
            }],
        }])
    }

    fn sample_contracts_failing(_repo_root: &Path) -> Result<Vec<Contract>, String> {
        fn fail_case(_: &RunContext) -> TestResult {
            TestResult::Fail(vec![Violation {
                contract_id: "DOCKER-999".to_string(),
                test_id: "docker.sample.fail".to_string(),
                file: Some("docker/images/runtime/Dockerfile".to_string()),
                line: Some(1),
                message: "sample failure".to_string(),
                evidence: Some("latest".to_string()),
            }])
        }
        Ok(vec![Contract {
            id: ContractId("DOCKER-999".to_string()),
            title: "sample fail",
            tests: vec![TestCase {
                id: TestId("docker.sample.fail".to_string()),
                title: "sample fail",
                kind: TestKind::Pure,
                run: fail_case,
            }],
        }])
    }

    #[test]
    fn pretty_output_is_stable() {
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", sample_contracts, Path::new("."), &options).expect("run");
        let pretty = to_pretty(&report);
        let expected = "Contracts: docker (mode=static)\nDOCKER-001 sample ....................................................... \u{1b}[32mPASS\u{1b}[0m\n  docker.sample.pass .................................................... \u{1b}[32mPASS\u{1b}[0m\nSummary: 1 contracts, 1 tests: 1 pass, 0 fail, 0 skip, 0 error\n";
        assert_eq!(pretty, expected);
    }

    #[test]
    fn json_serialization_contains_summary_and_tests() {
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", sample_contracts_failing, Path::new("."), &options).expect("run");
        let payload = to_json(&report);
        assert_eq!(payload["schema_version"], 1);
        assert_eq!(payload["summary"]["contracts"], 1);
        assert_eq!(payload["summary"]["tests"], 1);
        assert_eq!(payload["summary"]["fail"], 1);
        assert_eq!(
            payload["tests"][0]["violations"][0]["message"],
            "sample failure"
        );
    }

    #[test]
    fn panic_in_test_case_becomes_error_result() {
        fn panic_case(_: &RunContext) -> TestResult {
            panic!("boom");
        }
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            Ok(vec![Contract {
                id: ContractId("DOCKER-998".to_string()),
                title: "panic case",
                tests: vec![TestCase {
                    id: TestId("docker.sample.panic".to_string()),
                    title: "panic case",
                    kind: TestKind::Pure,
                    run: panic_case,
                }],
            }])
        }
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.exit_code(), 1);
    }

    #[test]
    fn effect_subprocess_requires_allow_subprocess_flag() {
        fn effect_case(_: &RunContext) -> TestResult {
            TestResult::Pass
        }
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            Ok(vec![Contract {
                id: ContractId("DOCKER-100".to_string()),
                title: "effect case",
                tests: vec![TestCase {
                    id: TestId("docker.effect.subprocess".to_string()),
                    title: "requires subprocess",
                    kind: TestKind::Subprocess,
                    run: effect_case,
                }],
            }])
        }
        let options = RunOptions {
            mode: Mode::Effect,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 30,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn effect_network_requires_allow_network_flag() {
        fn effect_case(_: &RunContext) -> TestResult {
            TestResult::Pass
        }
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            Ok(vec![Contract {
                id: ContractId("DOCKER-103".to_string()),
                title: "effect network",
                tests: vec![TestCase {
                    id: TestId("docker.effect.network".to_string()),
                    title: "requires network",
                    kind: TestKind::Network,
                    run: effect_case,
                }],
            }])
        }
        let options = RunOptions {
            mode: Mode::Effect,
            allow_subprocess: true,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 30,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.error_count(), 1);
    }
}
