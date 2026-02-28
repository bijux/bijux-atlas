#[test]
fn docker_build_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "build", "--format", "json"])
        .output()
        .expect("docker build");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker build requires --allow-subprocess"));
}

#[test]
fn docker_check_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "check", "--allow-subprocess", "--format", "json"])
        .output()
        .expect("docker check");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn docker_smoke_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "smoke", "--format", "json"])
        .output()
        .expect("docker smoke");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker smoke requires --allow-subprocess"));
}

#[test]
fn docker_scan_requires_allow_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "scan", "--allow-subprocess", "--format", "json"])
        .output()
        .expect("docker scan");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker scan requires --allow-network"));
}

#[test]
fn docker_policy_check_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "policy", "check", "--format", "json"])
        .output()
        .expect("docker policy check");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("action").and_then(|v| v.as_str()),
        Some("policy_check")
    );
}

#[test]
fn docker_lock_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "lock", "--format", "json"])
        .output()
        .expect("docker lock");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker lock requires --allow-write"));
}

#[test]
fn build_bin_requires_effect_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "bin", "--format", "json"])
        .output()
        .expect("build bin");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build bin requires --allow-subprocess"));
}

#[test]
#[ignore = "slow"]
fn build_bin_writes_manifest_when_effects_enabled() {
    let repo = repo_root();
    let manifest = repo.join("artifacts/dist/bin/manifest.json");
    let _ = fs::remove_file(&manifest);
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&repo)
        .args([
            "build",
            "bin",
            "--allow-subprocess",
            "--allow-write",
            "--format",
            "json",
            "--run-id",
            "build_bin_contract",
        ])
        .output()
        .expect("build bin");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload.get("action").and_then(|v| v.as_str()), Some("bin"));
    assert!(
        manifest.exists(),
        "manifest should exist: {}",
        manifest.display()
    );
    let manifest_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(manifest).expect("read manifest")).expect("manifest json");
    assert_eq!(
        manifest_payload.get("kind").and_then(|v| v.as_str()),
        Some("build_bin_manifest")
    );
}

#[test]
fn build_clean_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "clean", "--format", "json"])
        .output()
        .expect("build clean");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build clean requires --allow-write"));
}

#[test]
fn build_dist_requires_effect_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "dist", "--format", "json"])
        .output()
        .expect("build dist");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build dist requires --allow-subprocess"));
}

#[test]
fn build_doctor_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "doctor", "--format", "json"])
        .output()
        .expect("build doctor");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("action").and_then(|v| v.as_str()),
        Some("doctor")
    );
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn build_plan_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "plan", "--format", "json"])
        .output()
        .expect("build plan");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    assert_eq!(payload.get("action").and_then(|v| v.as_str()), Some("plan"));
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn build_verify_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "verify", "--format", "json"])
        .output()
        .expect("build verify");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build verify requires --allow-subprocess"));
}

#[test]
fn build_meta_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "meta", "--format", "json"])
        .output()
        .expect("build meta");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build meta requires --allow-write"));
}
