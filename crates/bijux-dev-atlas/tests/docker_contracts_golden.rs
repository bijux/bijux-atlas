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
    fs::create_dir_all(base.join("ops/policy")).expect("mkdir ops policy");
    fs::write(
        base.join("docker/images/runtime/Dockerfile"),
        dockerfile_text,
    )
    .expect("write dockerfile");
    fs::write(base.join("docker/README.md"), "# docker\n").expect("write docker readme");
    fs::write(base.join("Cargo.toml"), "[workspace]\n").expect("write cargo");
    fs::write(
        base.join("ops/policy/required-contracts.json"),
        serde_json::json!({
            "schema_version": 1,
            "contracts": [
                {
                    "domain": "docker",
                    "contract_id": "DOCKER-000",
                    "lanes": ["pr", "merge", "release"],
                    "owner": "atlas-maintainers",
                    "rationale": "fixture policy entry"
                }
            ]
        })
        .to_string(),
    )
    .expect("write required contracts policy");
    fs::write(
        base.join("docker/policy.json"),
        serde_json::json!({
            "schema_version": 1,
            "allow_tagged_images_exceptions": [],
            "allow_platform_in_from": false,
            "shell_policy": "forbid",
            "allow_root_runtime_images": [],
            "allow_add_exceptions": [],
            "allow_secret_copy_patterns": [],
            "profiles": {
                "local": {"allow_scan_skip": true},
                "ci": {"allow_scan_skip": false}
            },
            "runtime_engine": "docker",
            "airgap_build": {"declared": false, "policy": "stub"},
            "multi_registry_push": {"declared": false, "registries": []},
            "downloaded_assets": {"require_digest_pins": true},
            "vendored_binaries": {"allow": []},
            "required_oci_labels": [
                "org.opencontainers.image.source",
                "org.opencontainers.image.version",
                "org.opencontainers.image.revision",
                "org.opencontainers.image.created",
                "org.opencontainers.image.ref.name",
                "org.opencontainers.image.licenses"
            ]
        })
        .to_string(),
    )
    .expect("write policy");
    fs::write(base.join(".dockerignore"), ".git\nartifacts\ntarget\n").expect("dockerignore");
    fs::write(
        base.join("docker/bases.lock"),
        serde_json::json!({
            "schema_version": 1,
            "images": [
                {"name": "builder", "image": "rust:1.84.1-bookworm", "digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
                {"name": "runtime", "image": "gcr.io/distroless/cc-debian12:nonroot", "digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
            ]
        })
        .to_string(),
    )
    .expect("bases");
    fs::write(
        base.join("docker/images.manifest.json"),
        serde_json::json!({
            "schema_version": 1,
            "images": [{"name": "runtime", "dockerfile": "docker/images/runtime/Dockerfile", "context": ".", "smoke": ["/app/bijux-atlas", "version"]}]
        })
        .to_string(),
    )
    .expect("manifest");
    #[cfg(unix)]
    std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", base.join("Dockerfile"))
        .expect("symlink root dockerfile");
    bijux_dev_atlas::contracts::docker::sync_contract_markdown(base)
        .expect("sync contract markdown");
    bijux_dev_atlas::contracts::docker::sync_contract_registry_json(base)
        .expect("sync contract registry");
    bijux_dev_atlas::contracts::docker::sync_contract_gate_map_json(base)
        .expect("sync contract gate map");
}

fn run_for_single_test(repo_root: &Path, contract_id: &str, test_id: &str) -> serde_json::Value {
    let report = bijux_dev_atlas::contracts::run(
        "docker",
        bijux_dev_atlas::contracts::docker::contracts,
        repo_root,
        &bijux_dev_atlas::contracts::RunOptions {
            lane: bijux_dev_atlas::contracts::ContractLane::Local,
            mode: bijux_dev_atlas::contracts::Mode::Static,
            required_only: false,
            ci: false,
            color_enabled: false,
            allow_subprocess: false,
            allow_network: false,
            allow_k8s: false,
            allow_fs_write: false,
            allow_docker_daemon: false,
            deny_skip_required: true,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: Some(contract_id.to_string()),
            test_filter: Some(test_id.to_string()),
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
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
