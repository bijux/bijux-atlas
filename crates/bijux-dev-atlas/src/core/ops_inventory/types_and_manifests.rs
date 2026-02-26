
use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;

const OPS_STACK_PROFILES_PATH: &str = "ops/stack/profiles.json";
const OPS_STACK_VERSION_MANIFEST_PATH: &str = "ops/stack/generated/version-manifest.json";
const OPS_TOOLCHAIN_PATH: &str = "ops/inventory/toolchain.json";
const OPS_PINS_PATH: &str = "ops/inventory/pins.yaml";
const OPS_SURFACES_PATH: &str = "ops/inventory/surfaces.json";
const OPS_MIRROR_POLICY_PATH: &str = "ops/inventory/generated-committed-mirror.json";
const OPS_CONTRACTS_PATH: &str = "ops/inventory/contracts.json";
const OPS_GATES_PATH: &str = "ops/inventory/gates.json";
const OPS_DATASETS_MANIFEST_PATH: &str = "ops/datasets/manifest.json";
const OPS_K8S_INSTALL_MATRIX_PATH: &str = "ops/k8s/install-matrix.json";
const OPS_K8S_CHART_PATH: &str = "ops/k8s/charts/bijux-atlas/Chart.yaml";
const OPS_OBSERVE_ALERT_CATALOG_PATH: &str = "ops/observe/alert-catalog.json";
const OPS_OBSERVE_SLO_DEFINITIONS_PATH: &str = "ops/observe/slo-definitions.json";
const OPS_OBSERVE_TELEMETRY_DRILLS_PATH: &str = "ops/observe/telemetry-drills.json";
const OPS_OBSERVE_READINESS_PATH: &str = "ops/observe/readiness.json";
const OPS_OBSERVE_TELEMETRY_INDEX_PATH: &str = "ops/observe/generated/telemetry-index.json";
const OPS_DATASETS_MANIFEST_LOCK_PATH: &str = "ops/datasets/manifest.lock";
const OPS_DATASETS_PROMOTION_RULES_PATH: &str = "ops/datasets/promotion-rules.json";
const OPS_DATASETS_QC_METADATA_PATH: &str = "ops/datasets/qc-metadata.json";
const OPS_DATASETS_FIXTURE_POLICY_PATH: &str = "ops/datasets/fixture-policy.json";
const OPS_DATASETS_ROLLBACK_POLICY_PATH: &str = "ops/datasets/rollback-policy.json";
const OPS_DATASETS_INDEX_PATH: &str = "ops/datasets/generated/dataset-index.json";
const OPS_DATASETS_LINEAGE_PATH: &str = "ops/datasets/generated/dataset-lineage.json";
const OPS_E2E_SUITES_PATH: &str = "ops/e2e/suites/suites.json";
const OPS_E2E_SCENARIOS_PATH: &str = "ops/e2e/scenarios/scenarios.json";
const OPS_E2E_EXPECTATIONS_PATH: &str = "ops/e2e/expectations/expectations.json";
const OPS_E2E_FIXTURE_ALLOWLIST_PATH: &str = "ops/e2e/fixtures/allowlist.json";
const OPS_E2E_REPRODUCIBILITY_POLICY_PATH: &str = "ops/e2e/reproducibility-policy.json";
const OPS_E2E_TAXONOMY_PATH: &str = "ops/e2e/taxonomy.json";
const OPS_E2E_SUMMARY_PATH: &str = "ops/e2e/generated/e2e-summary.json";
const OPS_E2E_COVERAGE_MATRIX_PATH: &str = "ops/e2e/generated/coverage-matrix.json";
const OPS_REPORT_SCHEMA_PATH: &str = "ops/report/schema.json";
const OPS_REPORT_EVIDENCE_LEVELS_PATH: &str = "ops/report/evidence-levels.json";
const OPS_REPORT_EXAMPLE_PATH: &str = "ops/report/examples/unified-report-example.json";
const OPS_REPORT_READINESS_SCORE_PATH: &str = "ops/report/generated/readiness-score.json";
const OPS_REPORT_DIFF_PATH: &str = "ops/report/generated/report-diff.json";
const OPS_REPORT_HISTORY_PATH: &str = "ops/report/generated/historical-comparison.json";
const OPS_REPORT_RELEASE_BUNDLE_PATH: &str = "ops/report/generated/release-evidence-bundle.json";
const OPS_LOAD_SUITES_MANIFEST_PATH: &str = "ops/load/suites/suites.json";
const OPS_LOAD_QUERY_LOCK_PATH: &str = "ops/load/queries/pinned-v1.lock";
const OPS_LOAD_SEED_POLICY_PATH: &str = "ops/load/contracts/deterministic-seed-policy.json";
const OPS_LOAD_QUERY_PACK_CATALOG_PATH: &str = "ops/load/contracts/query-pack-catalog.json";
const OPS_LOAD_SUMMARY_PATH: &str = "ops/load/generated/load-summary.json";
const OPS_LOAD_DRIFT_REPORT_PATH: &str = "ops/load/generated/load-drift-report.json";

const EXPECTED_TOOLCHAIN_SCHEMA: u64 = 1;
const EXPECTED_SURFACES_SCHEMA: u64 = 2;
const EXPECTED_MIRROR_SCHEMA: u64 = 1;
const EXPECTED_CONTRACTS_SCHEMA: u64 = 1;
const EXPECTED_STACK_PROFILES_SCHEMA: u64 = 1;
const EXPECTED_STACK_VERSION_SCHEMA: u64 = 1;
const EXPECTED_PINS_SCHEMA: u64 = 1;

const INVENTORY_INPUTS: [&str; 8] = [
    OPS_STACK_PROFILES_PATH,
    OPS_STACK_VERSION_MANIFEST_PATH,
    OPS_TOOLCHAIN_PATH,
    OPS_PINS_PATH,
    OPS_GATES_PATH,
    OPS_SURFACES_PATH,
    OPS_MIRROR_POLICY_PATH,
    OPS_CONTRACTS_PATH,
];

#[derive(Debug, Clone)]
struct CacheEntry {
    fingerprint: u64,
    inventory: OpsInventory,
}

static OPS_INVENTORY_CACHE: OnceLock<Mutex<HashMap<PathBuf, CacheEntry>>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
pub struct OpsInventory {
    pub stack_profiles: StackProfilesManifest,
    pub stack_version_manifest: StackVersionManifest,
    pub toolchain: ToolchainManifest,
    pub surfaces: SurfacesManifest,
    pub mirror_policy: MirrorPolicyManifest,
    pub contracts: ContractsManifest,
}

impl OpsInventory {
    pub fn load_and_validate(ops_root: &Path) -> Result<Self, String> {
        let repo_root = ops_root
            .parent()
            .ok_or_else(|| "ops root must be under repo root".to_string())?;
        let errors = validate_ops_inventory(repo_root);
        if !errors.is_empty() {
            return Err(errors.join("; "));
        }
        load_ops_inventory_cached(repo_root)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackVersionManifest {
    pub schema_version: u64,
    #[serde(flatten)]
    pub components: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackProfilesManifest {
    pub schema_version: u64,
    pub profiles: Vec<StackProfile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackProfile {
    pub name: String,
    pub kind_profile: String,
    pub cluster_config: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolchainManifest {
    pub schema_version: u64,
    pub images: BTreeMap<String, String>,
    pub tools: BTreeMap<String, ToolSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolSpec {
    pub required: bool,
    pub version_regex: String,
    pub probe_argv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SurfacesManifest {
    pub schema_version: u64,
    pub actions: Vec<SurfaceAction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SurfaceAction {
    pub id: String,
    pub domain: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MirrorPolicyManifest {
    pub schema_version: u64,
    pub mirrors: Vec<MirrorEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MirrorEntry {
    pub committed: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractsManifest {
    pub schema_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct GatesManifest {
    pub schema_version: u64,
    #[serde(default)]
    pub gates: Vec<GateEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct GateEntry {
    pub id: String,
    pub action_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PinsManifest {
    pub schema_version: u64,
    #[serde(default)]
    pub images: BTreeMap<String, String>,
    #[serde(default)]
    pub dataset_ids: Vec<String>,
    #[serde(default)]
    pub versions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetsManifest {
    pub schema_version: u64,
    #[serde(default)]
    pub datasets: Vec<DatasetEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetEntry {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetManifestLock {
    pub schema_version: u64,
    #[serde(default)]
    pub entries: Vec<DatasetManifestLockEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetManifestLockEntry {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetPromotionRules {
    pub schema_version: u64,
    pub pins_source: String,
    pub manifest_lock: String,
    #[serde(default)]
    pub environments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetQcMetadata {
    pub schema_version: u64,
    pub stale_after_days: u64,
    pub golden_summary: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetFixturePolicy {
    pub schema_version: u64,
    pub allow_remote_download: bool,
    #[serde(default)]
    pub fixture_roots: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetRollbackPolicy {
    pub schema_version: u64,
    pub strategy: String,
    #[serde(default)]
    pub rollback_steps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetIndex {
    pub schema_version: u64,
    #[serde(default)]
    pub dataset_ids: Vec<String>,
    #[serde(default)]
    pub missing_dataset_ids: Vec<String>,
    #[serde(default)]
    pub stale_dataset_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetLineage {
    pub schema_version: u64,
    #[serde(default)]
    pub nodes: Vec<DatasetLineageNode>,
    #[serde(default)]
    pub edges: Vec<DatasetLineageEdge>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetLineageNode {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetLineageEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
struct K8sInstallMatrix {
    pub schema_version: u64,
    pub profiles: Vec<K8sInstallProfile>,
}

#[derive(Debug, Clone, Deserialize)]
struct K8sInstallProfile {
    pub name: String,
    pub values_file: String,
    pub suite: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ObserveCatalog {
    pub schema_version: u64,
    #[serde(default)]
    pub alerts: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ObserveSloDefinitions {
    pub schema_version: u64,
    #[serde(default)]
    pub slos: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ObserveDrillCatalog {
    pub schema_version: u64,
    #[serde(default)]
    pub drills: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ObserveReadiness {
    pub schema_version: u64,
    pub status: String,
    #[serde(default)]
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ObserveTelemetryIndex {
    pub schema_version: u64,
    #[serde(default)]
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadSuitesManifest {
    pub schema_version: u64,
    pub query_set: String,
    pub scenarios_dir: String,
    #[serde(default)]
    pub suites: Vec<LoadSuiteEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadSuiteEntry {
    pub name: String,
    #[serde(default)]
    pub kind: String,
    pub scenario: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadSeedPolicy {
    pub schema_version: u64,
    pub deterministic_seed: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadQueryPackCatalog {
    pub schema_version: u64,
    #[serde(default)]
    pub packs: Vec<LoadQueryPack>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadQueryPack {
    pub id: String,
    pub query_file: String,
    pub lock_file: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadSummary {
    pub schema_version: u64,
    #[serde(default)]
    pub suites: Vec<String>,
    pub deterministic_seed: u64,
    #[serde(default)]
    pub query_pack: String,
    pub scenario_coverage: LoadScenarioCoverage,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadScenarioCoverage {
    #[serde(default)]
    pub covered: Vec<String>,
    #[serde(default)]
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadDriftReport {
    pub schema_version: u64,
    pub status: String,
    #[serde(default)]
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadQueryLock {
    pub schema_version: u64,
    pub source: String,
    pub file_sha256: String,
    #[serde(default)]
    pub query_hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eSuitesManifest {
    pub schema_version: u64,
    #[serde(default)]
    pub suites: Vec<E2eSuite>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eSuite {
    pub id: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eScenariosManifest {
    pub schema_version: u64,
    #[serde(default)]
    pub scenarios: Vec<E2eScenario>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eScenario {
    pub id: String,
    pub action_id: Option<String>,
    #[serde(default)]
    pub compose: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eExpectations {
    pub schema_version: u64,
    #[serde(default)]
    pub expectations: Vec<E2eExpectation>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eExpectation {
    pub scenario_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eFixtureAllowlist {
    pub schema_version: u64,
    #[serde(default)]
    pub allowed_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eReproducibilityPolicy {
    pub schema_version: u64,
    pub ordering: String,
    pub seed_source: String,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eTaxonomy {
    pub schema_version: u64,
    #[serde(default)]
    pub categories: Vec<E2eCategory>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eCategory {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eSummary {
    pub schema_version: u64,
    pub status: String,
    pub suite_count: u64,
    pub scenario_count: u64,
    #[serde(default)]
    pub suite_ids: Vec<String>,
    #[serde(default)]
    pub scenario_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eCoverageMatrix {
    pub schema_version: u64,
    #[serde(default)]
    pub rows: Vec<E2eCoverageRow>,
    #[serde(default)]
    pub missing_domains: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct E2eCoverageRow {
    pub scenario_id: String,
    #[serde(default)]
    pub covers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportEvidenceLevels {
    pub schema_version: u64,
    #[serde(default)]
    pub levels: Vec<ReportEvidenceLevel>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportEvidenceLevel {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportReadinessScore {
    pub schema_version: u64,
    pub score: u64,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportDiff {
    pub schema_version: u64,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportHistoricalComparison {
    pub schema_version: u64,
    pub status: String,
    pub trend: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportReleaseEvidenceBundle {
    pub schema_version: u64,
    pub status: String,
    #[serde(default)]
    pub bundle_paths: Vec<String>,
}

