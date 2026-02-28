// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use serde_yaml::Value as YamlValue;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&read(path))
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn load_yaml(path: &Path) -> YamlValue {
    serde_yaml::from_str(&read(path))
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

#[test]
fn chart_values_profiles_use_only_schema_declared_top_level_keys() {
    let root = repo_root();
    let schema = load_json(&root.join("ops/k8s/charts/bijux-atlas/values.schema.json"));
    let allowed = schema["properties"]
        .as_object()
        .expect("schema properties")
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let matrix = load_json(&root.join("ops/k8s/install-matrix.json"));
    let profiles = matrix["profiles"].as_array().expect("profiles");
    let mut violations = Vec::new();

    for profile in profiles {
        let values_file = profile["values_file"].as_str().expect("values file");
        let yaml = load_yaml(&root.join(values_file));
        let map = yaml.as_mapping().expect("values mapping");
        for key in map.keys().filter_map(YamlValue::as_str) {
            if !allowed.contains(key) {
                violations.push(format!("{values_file} uses unknown top-level key `{key}`"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "k8s values profiles must stay within the declared schema surface:\n{}",
        violations.join("\n")
    );
}

#[test]
fn deployment_template_enforces_image_pull_security_and_mount_contracts() {
    let root = repo_root();
    let text = read(&root.join("ops/k8s/charts/bijux-atlas/templates/deployment.yaml"));
    for required in [
        "imagePullPolicy:",
        "securityContext:",
        "envFrom:",
        "configMapRef:",
        "ATLAS_CACHE_ROOT",
        "volumeMounts:",
        "mountPath: /cache",
        "mountPath: /tmp",
        "readinessProbe:",
        "livenessProbe:",
        "resources:",
    ] {
        assert!(
            text.contains(required),
            "deployment template missing `{required}`"
        );
    }
}

#[test]
fn chart_templates_keep_service_and_port_contracts_in_sync() {
    let root = repo_root();
    let deployment = read(&root.join("ops/k8s/charts/bijux-atlas/templates/deployment.yaml"));
    let service = read(&root.join("ops/k8s/charts/bijux-atlas/templates/service.yaml"));
    assert!(
        deployment.contains("containerPort: {{ .Values.service.port }}"),
        "deployment template must source container port from .Values.service.port"
    );
    assert!(
        service.contains("port: {{ .Values.service.port }}"),
        "service template must source service port from .Values.service.port"
    );
    assert!(
        service.contains("targetPort: http"),
        "service template must target the named http container port"
    );
}

#[test]
fn service_account_and_rbac_expectations_are_explicit() {
    let root = repo_root();
    let templates_dir = root.join("ops/k8s/charts/bijux-atlas/templates");
    let deployment = read(&templates_dir.join("deployment.yaml"));
    let template_names = fs::read_dir(&templates_dir)
        .expect("templates dir")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<BTreeSet<_>>();

    if deployment.contains("serviceAccountName:") {
        assert!(
            template_names.contains("serviceaccount.yaml"),
            "serviceaccount template is required when deployment declares serviceAccountName"
        );
    }

    let has_rbac_template = template_names.iter().any(|name| name.contains("role"));
    if has_rbac_template {
        assert!(
            template_names.iter().any(|name| name.contains("binding")),
            "rbac bindings must exist when role templates are declared"
        );
    }
}

#[test]
fn deployment_template_rejects_invalid_image_reference_shape() {
    let root = repo_root();
    let deployment = read(&root.join("ops/k8s/charts/bijux-atlas/templates/deployment.yaml"));
    assert!(
        deployment.contains("fail \"image.repository must not include '@'; use image.digest for digests\""),
        "deployment template must reject repository values containing digests"
    );
}

#[test]
fn helm_lint_passes_for_the_canonical_chart() {
    let root = repo_root();
    let output = Command::new("helm")
        .current_dir(&root)
        .args([
            "lint",
            "ops/k8s/charts/bijux-atlas",
            "-f",
            "ops/k8s/charts/bijux-atlas/values.yaml",
        ])
        .output()
        .expect("helm lint");
    assert!(
        output.status.success(),
        "helm lint must pass:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
