// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn write_repo_with_dockerfile(base: &Path, dockerfile_text: &str) {
    fs::create_dir_all(base.join("docker/images/runtime")).expect("mkdir docker runtime");
    fs::write(
        base.join("docker/images/runtime/Dockerfile"),
        dockerfile_text,
    )
    .expect("write dockerfile");
    fs::write(base.join("docker/README.md"), "# docker\n").expect("write docker readme");
    fs::write(base.join("Cargo.toml"), "[workspace]\n").expect("write cargo");
    fs::write(
        base.join("docker/policy.json"),
        serde_json::json!({
            "schema_version": 1,
            "allow_tagged_images_exceptions": [],
            "required_oci_labels": [
                "org.opencontainers.image.source",
                "org.opencontainers.image.version",
                "org.opencontainers.image.revision",
                "org.opencontainers.image.created",
                "org.opencontainers.image.ref.name"
            ]
        })
        .to_string(),
    )
    .expect("write policy");
    #[cfg(unix)]
    std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", base.join("Dockerfile"))
        .expect("symlink root dockerfile");
    bijux_dev_atlas::contracts::docker::sync_contract_markdown(base)
        .expect("sync contract markdown");
}

fn run_for_single_test(repo_root: &Path, contract_id: &str, test_id: &str) -> serde_json::Value {
    let report = bijux_dev_atlas::contracts::run(
        "docker",
        bijux_dev_atlas::contracts::docker::contracts,
        repo_root,
        &bijux_dev_atlas::contracts::RunOptions {
            mode: bijux_dev_atlas::contracts::Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: Some(contract_id.to_string()),
            test_filter: Some(test_id.to_string()),
            list_only: false,
            artifacts_root: None,
        },
    )
    .expect("run contracts");

    let payload = bijux_dev_atlas::contracts::to_json(&report);
    let tests = payload["tests"].as_array().expect("tests array");
    let violations = tests
        .iter()
        .flat_map(|row| {
            row["violations"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
        })
        .collect::<Vec<_>>();
    serde_json::Value::Array(violations)
}

#[test]
fn latest_tag_violation_matches_golden() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dockerfile = include_str!("fixtures/dockerfiles/fail_latest.Dockerfile");
    write_repo_with_dockerfile(tmp.path(), dockerfile);

    let actual = run_for_single_test(tmp.path(), "DOCKER-006", "docker.from.no_latest");
    let expected =
        serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(workspace_root().join(
                "crates/bijux-dev-atlas/tests/goldens/docker_contracts_latest_violation.json",
            ))
            .expect("read golden"),
        )
        .expect("parse golden");

    assert_eq!(actual, expected);
}

#[test]
fn digest_violation_matches_golden() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dockerfile = include_str!("fixtures/dockerfiles/fail_missing_digest.Dockerfile");
    write_repo_with_dockerfile(tmp.path(), dockerfile);

    let actual = run_for_single_test(tmp.path(), "DOCKER-007", "docker.from.digest_required");
    let expected =
        serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(workspace_root().join(
                "crates/bijux-dev-atlas/tests/goldens/docker_contracts_digest_violation.json",
            ))
            .expect("read golden"),
        )
        .expect("parse golden");

    assert_eq!(actual, expected);
}
