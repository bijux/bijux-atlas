// SPDX-License-Identifier: Apache-2.0

use std::fs;

use bijux_dev_atlas::engine::{
    ArtifactStore, CommandRunnableExecutor, EffectPolicy, RunStatus, RunnableExecutor,
    RunnableRunContext,
};
use bijux_dev_atlas::model::{
    RunId, RunnableEntry, RunnableId, RunnableKind, RunnableMode, SuiteId,
};
use bijux_dev_atlas::registry::RunnableRegistry;
use bijux_dev_atlas::runtime::{Capabilities, FakeWorld};

#[test]
fn runnable_registry_loads_checks_and_contracts() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf();
    let registry = RunnableRegistry::load(&repo_root).expect("load runnable registry");
    assert!(
        registry
            .all()
            .iter()
            .any(|entry| entry.kind == RunnableKind::Check),
        "expected at least one check runnable"
    );
    assert!(
        registry
            .all()
            .iter()
            .any(|entry| entry.kind == RunnableKind::Contract),
        "expected at least one contract runnable"
    );
    assert!(
        registry
            .suites()
            .iter()
            .any(|suite| suite.id.as_str() == "checks"),
        "expected checks suite"
    );
    assert!(
        registry
            .suites()
            .iter()
            .any(|suite| suite.id.as_str() == "contracts"),
        "expected contracts suite"
    );
}

#[test]
fn command_executor_writes_skip_report_for_missing_tool() {
    let temp_root =
        std::env::temp_dir().join(format!("bijux-dev-atlas-runnable-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&temp_root).expect("create temp root");

    let entry = RunnableEntry {
        id: RunnableId::parse("test.skip.missing-tool").expect("id"),
        suite: SuiteId::parse("checks").expect("suite"),
        kind: RunnableKind::Check,
        mode: RunnableMode::Pure,
        summary: "test skip".to_string(),
        owner: "tests".to_string(),
        group: "tests".to_string(),
        tags: Vec::new(),
        commands: vec!["cargo --version".to_string()],
        report_ids: vec!["run-result".to_string()],
        reports: vec!["run-result.json".to_string()],
        required_tools: vec!["tool-that-does-not-exist".to_string()],
        missing_tools_policy: "skip".to_string(),
        effects_required: Vec::new(),
    };
    let world = FakeWorld::default();
    let executor = CommandRunnableExecutor {
        process: &world,
        fs: &world,
    };
    let context = RunnableRunContext {
        repo_root: temp_root.clone(),
        run_id: RunId::from_seed("skip-run"),
        artifact_store: ArtifactStore::new(temp_root.join("artifacts/run")),
        effect_policy: EffectPolicy {
            capabilities: Capabilities {
                fs_write: true,
                subprocess: true,
                git: true,
                network: true,
            },
        },
    };
    let result = executor.execute(&entry, &context).expect("execute");
    assert_eq!(result.status, RunStatus::Skip);
    assert_eq!(result.report_refs.len(), 1);
    assert!(fs::read_to_string(&result.report_refs[0].path)
        .expect("read report")
        .contains("\"status\": \"skip\""));
}
