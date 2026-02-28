use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EffectKind {
    Subprocess,
    Network,
    K8s,
    FsWrite,
    DockerDaemon,
}

impl EffectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Subprocess => "subprocess",
            Self::Network => "network",
            Self::K8s => "k8s",
            Self::FsWrite => "fs-write",
            Self::DockerDaemon => "docker-daemon",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractMode {
    Static,
    Effect,
    Both,
}

impl ContractMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Effect => "effect",
            Self::Both => "both",
        }
    }
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
    pub allow_k8s: bool,
    pub allow_fs_write: bool,
    pub allow_docker_daemon: bool,
    pub skip_missing_tools: bool,
    pub timeout_seconds: u64,
}

pub struct RunOptions {
    pub mode: Mode,
    pub allow_subprocess: bool,
    pub allow_network: bool,
    pub allow_k8s: bool,
    pub allow_fs_write: bool,
    pub allow_docker_daemon: bool,
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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
            Self::Error => "ERROR",
        }
    }

    pub fn as_colored(self) -> &'static str {
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
    pub mode: ContractMode,
    pub effects: Vec<EffectKind>,
    pub status: CaseStatus,
}

pub struct RunMetadata {
    pub run_id: String,
    pub commit_sha: Option<String>,
    pub dirty_tree: bool,
}

pub struct RunReport {
    pub domain: String,
    pub mode: Mode,
    pub metadata: RunMetadata,
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

pub struct CoverageReport {
    pub group: String,
    pub contracts: usize,
    pub tests: usize,
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    pub error: usize,
}

pub struct EffectRequirement {
    pub allow_subprocess: bool,
    pub allow_network: bool,
    pub allow_k8s: bool,
    pub allow_fs_write: bool,
    pub allow_docker_daemon: bool,
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
        if self.error_count() > 0 || self.fail_count() > 0 {
            1
        } else {
            0
        }
    }
}

pub fn run_metadata(repo_root: &Path) -> RunMetadata {
    let repo_display = repo_root.display().to_string();
    let run_id = std::env::var("RUN_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "local".to_string());
    let commit_sha = Command::new("git")
        .args(["-C", &repo_display, "rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty());
    let dirty_tree = Command::new("git")
        .args(["-C", &repo_display, "status", "--porcelain"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);
    RunMetadata {
        run_id,
        commit_sha,
        dirty_tree,
    }
}

pub fn maturity_score(contracts: &[ContractSummary]) -> serde_json::Value {
    let total = contracts.len().max(1) as f64;
    let mapped_gates = contracts.len() as f64;
    let explain = contracts.len() as f64;
    let json_schema = contracts.len() as f64;
    let effect_safety = contracts
        .iter()
        .filter(|contract| {
            contract.mode == ContractMode::Static
                || contract
                    .effects
                    .iter()
                    .all(|effect| matches!(effect, EffectKind::Subprocess | EffectKind::Network))
        })
        .count() as f64;
    serde_json::json!({
        "mapped_gates_pct": ((mapped_gates / total) * 100.0).round() as u64,
        "explain_pct": ((explain / total) * 100.0).round() as u64,
        "json_schema_pct": ((json_schema / total) * 100.0).round() as u64,
        "effect_safety_pct": ((effect_safety / total) * 100.0).round() as u64,
    })
}

pub fn registry_snapshot(domain: &str, contracts: &[Contract]) -> Vec<RegistrySnapshotRow> {
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
    let mut lints = Vec::new();
    let id_re = match regex::Regex::new(r"^[A-Z]+(?:-[A-Z0-9]+)*-[0-9]{3,}$") {
        Ok(value) => value,
        Err(err) => {
            lints.push(RegistryLint {
                code: "internal-regex",
                message: format!("contract id regex failed to compile: {err}"),
            });
            return lints;
        }
    };
    let test_id_re = match regex::Regex::new(r"^[a-z0-9]+(?:\.[a-z0-9_]+)+$") {
        Ok(value) => value,
        Err(err) => {
            lints.push(RegistryLint {
                code: "internal-regex",
                message: format!("test id regex failed to compile: {err}"),
            });
            return lints;
        }
    };
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
            .split_whitespace()
            .filter(|word| {
                let word = word.to_ascii_lowercase();
                word != "contract" && word != "policy"
            })
            .collect::<Vec<_>>()
            .join(" ");
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

pub fn lint_contracts(catalogs: &[(&str, &[Contract])]) -> Vec<RegistryLint> {
    let mut lints = Vec::new();
    for (domain, contracts) in catalogs {
        for contract in *contracts {
            let mode = contract_mode(contract);
            let effects = contract_effects(contract);
            match mode {
                ContractMode::Static if !effects.is_empty() => lints.push(RegistryLint {
                    code: "static-effects",
                    message: format!(
                        "{}:{} derives static mode but exposes effect kinds",
                        domain, contract.id.0
                    ),
                }),
                ContractMode::Effect if effects.is_empty() => lints.push(RegistryLint {
                    code: "effect-missing-effects",
                    message: format!(
                        "{}:{} derives effect mode but has no effect kinds",
                        domain, contract.id.0
                    ),
                }),
                ContractMode::Both if effects.is_empty() => lints.push(RegistryLint {
                    code: "mixed-missing-effects",
                    message: format!(
                        "{}:{} derives mixed mode but has no effect kinds",
                        domain, contract.id.0
                    ),
                }),
                _ => {}
            }
        }
    }
    lints.sort_by(|a, b| a.code.cmp(b.code).then(a.message.cmp(&b.message)));
    lints
}

pub fn validate_registry(catalogs: &[(&str, &[Contract])]) -> Result<(), Vec<RegistryLint>> {
    let mut rows = Vec::new();
    for (domain, contracts) in catalogs {
        rows.extend(registry_snapshot(domain, contracts));
    }
    let mut lints = lint_registry_rows(&rows);
    lints.extend(lint_contracts(catalogs));
    lints.sort_by(|a, b| a.code.cmp(b.code).then(a.message.cmp(&b.message)));
    if lints.is_empty() { Ok(()) } else { Err(lints) }
}

pub fn coverage_report(report: &RunReport) -> CoverageReport {
    CoverageReport {
        group: report.domain.clone(),
        contracts: report.total_contracts(),
        tests: report.total_tests(),
        pass: report.pass_count(),
        fail: report.fail_count(),
        skip: report.skip_count(),
        error: report.error_count(),
    }
}
