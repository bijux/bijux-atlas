// SPDX-License-Identifier: Apache-2.0
//! Contracts runner engine.
//!
//! This module provides a domain-agnostic contracts runner with deterministic ordering,
//! filterable execution, pretty and JSON output, and explicit effect gating.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

pub mod docker;
pub mod make;
pub mod ops;

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
    pub only_contracts: Vec<String>,
    pub only_tests: Vec<String>,
    pub skip_contracts: Vec<String>,
    pub tags: Vec<String>,
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

pub struct RegistrySnapshotRow {
    pub domain: String,
    pub id: String,
    pub title: String,
    pub test_ids: Vec<String>,
}

pub struct RegistryLint {
    pub code: &'static str,
    pub message: String,
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

fn matches_any_filter(filters: &[String], value: &str) -> bool {
    filters.is_empty() || filters.iter().any(|filter| wildcard_match(filter, value))
}

fn matches_skip_filter(filters: &[String], value: &str) -> bool {
    !filters.is_empty() && filters.iter().any(|filter| wildcard_match(filter, value))
}

fn derived_contract_tags(contract: &Contract) -> BTreeSet<&'static str> {
    let mut tags = BTreeSet::from(["ci"]);
    let mut has_pure = false;
    let mut has_effect = false;
    for case in &contract.tests {
        match case.kind {
            TestKind::Pure => has_pure = true,
            TestKind::Subprocess | TestKind::Network => has_effect = true,
        }
    }
    if has_pure {
        tags.insert("static");
    }
    if has_effect {
        tags.insert("effect");
        tags.insert("local");
        tags.insert("slow");
    }
    tags
}

fn matches_tags(filters: &[String], contract: &Contract) -> bool {
    if filters.is_empty() {
        return true;
    }
    let tags = derived_contract_tags(contract);
    filters.iter().any(|filter| {
        tags.iter()
            .any(|tag| wildcard_match(&filter.to_ascii_lowercase(), tag))
    })
}

pub fn registry_snapshot(
    domain: &str,
    contracts: &[Contract],
) -> Vec<RegistrySnapshotRow> {
    let mut rows = contracts
        .iter()
        .map(|contract| {
            let mut test_ids = contract
                .tests
                .iter()
                .map(|case| case.id.0.clone())
                .collect::<Vec<_>>();
            test_ids.sort();
            RegistrySnapshotRow {
                domain: domain.to_string(),
                id: contract.id.0.clone(),
                title: contract.title.to_string(),
                test_ids,
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.domain.cmp(&b.domain).then(a.id.cmp(&b.id)));
    rows
}

pub fn lint_registry_rows(rows: &[RegistrySnapshotRow]) -> Vec<RegistryLint> {
    let id_re = regex::Regex::new(r"^[A-Z]+(?:-[A-Z0-9]+)*-[0-9]{3,}$").expect("valid regex");
    let test_id_re = regex::Regex::new(r"^[a-z0-9]+(?:\.[a-z0-9_]+)+$").expect("valid regex");
    let mut lints = Vec::new();
    let mut contract_ids = BTreeMap::<String, Vec<String>>::new();
    let mut test_ids = BTreeMap::<String, Vec<String>>::new();
    let mut normalized_titles = BTreeMap::<String, Vec<String>>::new();

    for row in rows {
        contract_ids
            .entry(row.id.clone())
            .or_default()
            .push(format!("{}:{}", row.domain, row.id));
        normalized_titles
            .entry(row.title.trim().to_ascii_lowercase())
            .or_default()
            .push(format!("{}:{}", row.domain, row.id));
        if row.test_ids.is_empty() {
            lints.push(RegistryLint {
                code: "empty-contract",
                message: format!("{} has no tests", row.id),
            });
        }
        if !id_re.is_match(&row.id) {
            lints.push(RegistryLint {
                code: "contract-id-format",
                message: format!("{} does not match required contract id format", row.id),
            });
        }
        let simplified_title = row
            .title
            .replace(" contract", "")
            .replace(" policy", "")
            .trim()
            .to_string();
        if simplified_title.is_empty() {
            lints.push(RegistryLint {
                code: "title-filler",
                message: format!("{} title collapses to filler words only", row.id),
            });
        }
        for test_id in &row.test_ids {
            test_ids
                .entry(test_id.clone())
                .or_default()
                .push(format!("{}:{}", row.id, test_id));
            if !test_id_re.is_match(test_id) {
                lints.push(RegistryLint {
                    code: "test-id-format",
                    message: format!("{test_id} does not use dotted namespace format"),
                });
            }
        }
    }

    for (contract_id, owners) in contract_ids {
        if owners.len() > 1 {
            lints.push(RegistryLint {
                code: "duplicate-contract-id",
                message: format!("duplicate contract id {contract_id}: {}", owners.join(", ")),
            });
        }
    }
    for (test_id, owners) in test_ids {
        if owners.len() > 1 {
            lints.push(RegistryLint {
                code: "duplicate-test-id",
                message: format!("duplicate test id {test_id}: {}", owners.join(", ")),
            });
        }
    }
    for (title, owners) in normalized_titles {
        if owners.len() > 1 {
            lints.push(RegistryLint {
                code: "duplicate-title",
                message: format!("duplicate contract title `{title}`: {}", owners.join(", ")),
            });
        }
    }

    lints.sort_by(|a, b| a.code.cmp(b.code).then(a.message.cmp(&b.message)));
    lints
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
        if !matches_filter(&options.contract_filter, &contract.id.0)
            || !matches_any_filter(&options.only_contracts, &contract.id.0)
            || matches_skip_filter(&options.skip_contracts, &contract.id.0)
            || !matches_tags(&options.tags, &contract)
        {
            continue;
        }
        let mut cases = contract.tests;
        cases.sort_by_key(|t| t.id.0.clone());
        let mut contract_status = CaseStatus::Pass;
        let mut has_case = false;
        for case in cases {
            if !matches_filter(&options.test_filter, &case.id.0)
                || !matches_any_filter(&options.only_tests, &case.id.0)
            {
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
        if options.fail_fast && matches!(contract_status, CaseStatus::Fail | CaseStatus::Error) {
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
    fn dotted_with_width(label: &str, status: &str, width: usize) -> String {
        let left = if label.len() >= width {
            label.to_string()
        } else {
            format!("{label} {}", ".".repeat(width - label.len()))
        };
        format!("{left} {status}")
    }
    fn dotted(label: &str, status: &str) -> String {
        const WIDTH: usize = 72;
        dotted_with_width(label, status, WIDTH)
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
        for case in report.cases.iter().filter(|c| c.contract_id == contract.id) {
            // Keep two-space indentation while preserving a shared status column with contract rows.
            out.push_str(&format!(
                "  {}\n",
                dotted_with_width(&case.test_id, case.status.as_colored(), 70)
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
    if report.mode == Mode::Static && report.skip_count() > 0 {
        out.push_str("Note: effect-only tests are skipped in static mode; use --mode effect with required allow flags.\n");
    }
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

pub fn to_json_all(reports: &[RunReport]) -> serde_json::Value {
    let contracts = reports.iter().map(RunReport::total_contracts).sum::<usize>();
    let tests = reports.iter().map(RunReport::total_tests).sum::<usize>();
    let pass = reports.iter().map(RunReport::pass_count).sum::<usize>();
    let fail = reports.iter().map(RunReport::fail_count).sum::<usize>();
    let skip = reports.iter().map(RunReport::skip_count).sum::<usize>();
    let error = reports.iter().map(RunReport::error_count).sum::<usize>();
    let exit_code = if error > 0 {
        1
    } else if fail > 0 {
        2
    } else {
        0
    };
    serde_json::json!({
        "schema_version": 1,
        "domain": "all",
        "summary": {
            "contracts": contracts,
            "tests": tests,
            "pass": pass,
            "fail": fail,
            "skip": skip,
            "error": error,
            "exit_code": exit_code
        },
        "domains": reports.iter().map(to_json).collect::<Vec<_>>()
    })
}

pub fn to_pretty_all(reports: &[RunReport]) -> String {
    let mut out = String::new();
    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&to_pretty(report));
    }
    let contracts = reports.iter().map(RunReport::total_contracts).sum::<usize>();
    let tests = reports.iter().map(RunReport::total_tests).sum::<usize>();
    let pass = reports.iter().map(RunReport::pass_count).sum::<usize>();
    let fail = reports.iter().map(RunReport::fail_count).sum::<usize>();
    let skip = reports.iter().map(RunReport::skip_count).sum::<usize>();
    let error = reports.iter().map(RunReport::error_count).sum::<usize>();
    out.push_str(&format!(
        "\nSummary: {} contracts, {} tests: {} pass, {} fail, {} skip, {} error\n",
        contracts, tests, pass, fail, skip, error
    ));
    out
}

pub fn to_github(reports: &[RunReport]) -> String {
    let mut out = to_pretty_all(reports);
    for report in reports {
        for case in &report.cases {
            match case.status {
                CaseStatus::Fail => {
                    for violation in &case.violations {
                        let file = violation.file.clone().unwrap_or_default();
                        let line = violation.line.unwrap_or(1);
                        out.push_str(&format!(
                            "::error file={},line={},title={}::{}\n",
                            file, line, case.test_id, violation.message
                        ));
                    }
                }
                CaseStatus::Error => {
                    out.push_str(&format!(
                        "::error title={}::{}\n",
                        case.test_id,
                        case.note.clone().unwrap_or_else(|| "error".to_string())
                    ));
                }
                CaseStatus::Skip => {
                    out.push_str(&format!(
                        "::notice title={}::{}\n",
                        case.test_id,
                        case.note.clone().unwrap_or_else(|| "skipped".to_string())
                    ));
                }
                CaseStatus::Pass => {}
            }
        }
    }
    out
}

pub fn to_junit_all(reports: &[RunReport]) -> Result<String, String> {
    let tests = reports.iter().map(RunReport::total_tests).sum::<usize>();
    let failures = reports.iter().map(RunReport::fail_count).sum::<usize>();
    let errors = reports.iter().map(RunReport::error_count).sum::<usize>();
    let skipped = reports.iter().map(RunReport::skip_count).sum::<usize>();
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuites tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\">",
        tests, failures, errors, skipped
    ));
    for report in reports {
        let suite = to_junit(report)?;
        let start = suite
            .find("<testsuite")
            .ok_or_else(|| "invalid junit suite".to_string())?;
        let end = suite
            .rfind("</testsuite>")
            .ok_or_else(|| "invalid junit suite".to_string())?;
        out.push_str(&suite[start..end + "</testsuite>".len()]);
    }
    out.push_str("</testsuites>\n");
    Ok(out)
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn to_junit(report: &RunReport) -> Result<String, String> {
    let tests = report.total_tests();
    let failures = report.fail_count();
    let errors = report.error_count();
    let skipped = report.skip_count();
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuites><testsuite name=\"contracts.{}\" tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\">",
        xml_escape(&report.domain),
        tests,
        failures,
        errors,
        skipped
    ));
    for case in &report.cases {
        out.push_str(&format!(
            "<testcase classname=\"{}\" name=\"{}\">",
            xml_escape(&case.contract_id),
            xml_escape(&case.test_id)
        ));
        match case.status {
            CaseStatus::Pass => {}
            CaseStatus::Skip => {
                let note = case.note.as_deref().unwrap_or("skipped");
                out.push_str(&format!("<skipped message=\"{}\"/>", xml_escape(note)));
            }
            CaseStatus::Error => {
                let note = case.note.as_deref().unwrap_or("error");
                out.push_str(&format!(
                    "<error message=\"{}\">{}</error>",
                    xml_escape(note),
                    xml_escape(note)
                ));
            }
            CaseStatus::Fail => {
                let detail = case
                    .violations
                    .iter()
                    .map(|v| {
                        let location = match (&v.file, v.line) {
                            (Some(file), Some(line)) => format!("{file}:{line}"),
                            (Some(file), None) => file.clone(),
                            _ => "unknown-location".to_string(),
                        };
                        format!("{}: {}", location, v.message)
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                out.push_str(&format!(
                    "<failure message=\"{}\">{}</failure>",
                    xml_escape(&detail),
                    xml_escape(&detail)
                ));
            }
        }
        out.push_str("</testcase>");
    }
    out.push_str("</testsuite></testsuites>\n");
    Ok(out)
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
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
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
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report =
            run("docker", sample_contracts_failing, Path::new("."), &options).expect("run");
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
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
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
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
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
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn fail_fast_stops_after_first_failing_contract() {
        fn fail_case(_: &RunContext) -> TestResult {
            TestResult::Fail(vec![Violation {
                contract_id: "DOCKER-001".to_string(),
                test_id: "docker.sample.fail".to_string(),
                file: None,
                line: None,
                message: "failed".to_string(),
                evidence: None,
            }])
        }
        fn pass_case(_: &RunContext) -> TestResult {
            TestResult::Pass
        }
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            Ok(vec![
                Contract {
                    id: ContractId("DOCKER-001".to_string()),
                    title: "first",
                    tests: vec![TestCase {
                        id: TestId("docker.sample.fail".to_string()),
                        title: "failing",
                        kind: TestKind::Pure,
                        run: fail_case,
                    }],
                },
                Contract {
                    id: ContractId("DOCKER-002".to_string()),
                    title: "second",
                    tests: vec![TestCase {
                        id: TestId("docker.sample.pass".to_string()),
                        title: "passing",
                        kind: TestKind::Pure,
                        run: pass_case,
                    }],
                },
            ])
        }
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 30,
            fail_fast: true,
            contract_filter: None,
            test_filter: None,
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.total_contracts(), 1);
        assert_eq!(report.total_tests(), 1);
        assert_eq!(report.fail_count(), 1);
    }

    #[test]
    fn wildcard_filters_contract_and_test_ids() {
        fn pass_case(_: &RunContext) -> TestResult {
            TestResult::Pass
        }
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            Ok(vec![
                Contract {
                    id: ContractId("OPS-ROOT-001".to_string()),
                    title: "root",
                    tests: vec![
                        TestCase {
                            id: TestId("ops.root.surface.allowed".to_string()),
                            title: "allowed",
                            kind: TestKind::Pure,
                            run: pass_case,
                        },
                        TestCase {
                            id: TestId("ops.root.surface.extra".to_string()),
                            title: "extra",
                            kind: TestKind::Pure,
                            run: pass_case,
                        },
                    ],
                },
                Contract {
                    id: ContractId("OPS-INV-001".to_string()),
                    title: "inventory",
                    tests: vec![TestCase {
                        id: TestId("ops.inventory.registry.exists".to_string()),
                        title: "exists",
                        kind: TestKind::Pure,
                        run: pass_case,
                    }],
                },
            ])
        }
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 30,
            fail_fast: false,
            contract_filter: Some("OPS-ROOT-*".to_string()),
            test_filter: Some("ops.root.surface.allow*".to_string()),
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report = run("ops", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.total_contracts(), 1);
        assert_eq!(report.total_tests(), 1);
        assert_eq!(report.contracts[0].id, "OPS-ROOT-001");
        assert_eq!(report.cases[0].test_id, "ops.root.surface.allowed");
    }

    #[test]
    fn run_sorts_contracts_and_tests_deterministically() {
        fn pass_case(_: &RunContext) -> TestResult {
            TestResult::Pass
        }
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            Ok(vec![
                Contract {
                    id: ContractId("OPS-ROOT-002".to_string()),
                    title: "second",
                    tests: vec![
                        TestCase {
                            id: TestId("ops.root.beta".to_string()),
                            title: "beta",
                            kind: TestKind::Pure,
                            run: pass_case,
                        },
                        TestCase {
                            id: TestId("ops.root.alpha".to_string()),
                            title: "alpha",
                            kind: TestKind::Pure,
                            run: pass_case,
                        },
                    ],
                },
                Contract {
                    id: ContractId("OPS-ROOT-001".to_string()),
                    title: "first",
                    tests: vec![TestCase {
                        id: TestId("ops.root.first".to_string()),
                        title: "first",
                        kind: TestKind::Pure,
                        run: pass_case,
                    }],
                },
            ])
        }
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 30,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report = run("ops", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.contracts[0].id, "OPS-ROOT-001");
        assert_eq!(report.contracts[1].id, "OPS-ROOT-002");
        let second_contract_tests = report
            .cases
            .iter()
            .filter(|case| case.contract_id == "OPS-ROOT-002")
            .map(|case| case.test_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            second_contract_tests,
            vec!["ops.root.alpha", "ops.root.beta"]
        );
    }

    #[test]
    fn fail_exit_code_is_two_and_error_exit_code_is_one() {
        let fail_report = RunReport {
            domain: "ops".to_string(),
            mode: Mode::Static,
            contracts: vec![ContractSummary {
                id: "OPS-ROOT-001".to_string(),
                title: "fail".to_string(),
                status: CaseStatus::Fail,
            }],
            cases: vec![CaseReport {
                contract_id: "OPS-ROOT-001".to_string(),
                contract_title: "fail".to_string(),
                test_id: "ops.root.fail".to_string(),
                test_title: "fail".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Fail,
                violations: Vec::new(),
                note: None,
            }],
        };
        let error_report = RunReport {
            domain: "ops".to_string(),
            mode: Mode::Static,
            contracts: vec![ContractSummary {
                id: "OPS-ROOT-002".to_string(),
                title: "error".to_string(),
                status: CaseStatus::Error,
            }],
            cases: vec![CaseReport {
                contract_id: "OPS-ROOT-002".to_string(),
                contract_title: "error".to_string(),
                test_id: "ops.root.error".to_string(),
                test_title: "error".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Error,
                violations: Vec::new(),
                note: Some("panic".to_string()),
            }],
        };
        assert_eq!(fail_report.exit_code(), 2);
        assert_eq!(error_report.exit_code(), 1);
    }

}
