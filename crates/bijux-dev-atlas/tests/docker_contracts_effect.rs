// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn effect_mode_contracts_run_when_enabled() {
    if std::env::var("RUN_EFFECT_CONTRACTS").ok().as_deref() != Some("1") {
        return;
    }
    let report = bijux_dev_atlas::contracts::run(
        "docker",
        bijux_dev_atlas::contracts::docker::contracts,
        &workspace_root(),
        &bijux_dev_atlas::contracts::RunOptions {
            mode: bijux_dev_atlas::contracts::Mode::Effect,
            allow_subprocess: true,
            allow_network: true,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: true,
            contract_filter: Some("DOCKER-10*".to_string()),
            test_filter: None,
            list_only: false,
            artifacts_root: Some(workspace_root().join("artifacts/effect-contract-tests")),
        },
    )
    .expect("run effect contracts");

    assert!(report.total_tests() > 0, "effect suite should execute tests");
}

#[test]
fn effect_mode_reports_docker_daemon_error_with_unreachable_host() {
    if std::env::var("RUN_EFFECT_CONTRACTS").ok().as_deref() != Some("1") {
        return;
    }
    let original = std::env::var("DOCKER_HOST").ok();
    std::env::set_var("DOCKER_HOST", "unix:///tmp/does-not-exist.sock");

    let report = bijux_dev_atlas::contracts::run(
        "docker",
        bijux_dev_atlas::contracts::docker::contracts,
        &workspace_root(),
        &bijux_dev_atlas::contracts::RunOptions {
            mode: bijux_dev_atlas::contracts::Mode::Effect,
            allow_subprocess: true,
            allow_network: false,
            skip_missing_tools: true,
            timeout_seconds: 60,
            fail_fast: true,
            contract_filter: Some("DOCKER-100".to_string()),
            test_filter: Some("docker.build.runtime_image".to_string()),
            list_only: false,
            artifacts_root: Some(workspace_root().join("artifacts/effect-contract-tests")),
        },
    )
    .expect("run effect contracts with unreachable docker host");

    if let Some(val) = original {
        std::env::set_var("DOCKER_HOST", val);
    } else {
        std::env::remove_var("DOCKER_HOST");
    }

    assert!(report.fail_count() + report.error_count() > 0);
}
