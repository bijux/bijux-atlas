// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&read(path))
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn load_yaml(path: &Path) -> YamlValue {
    serde_yaml::from_str(&read(path))
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn install_matrix_values_files(root: &Path) -> Vec<String> {
    let matrix = load_json(&root.join("ops/k8s/install-matrix.json"));
    matrix["profiles"]
        .as_array()
        .expect("profiles")
        .iter()
        .map(|profile| {
            profile["values_file"]
                .as_str()
                .expect("values file")
                .to_string()
        })
        .collect()
}

fn render_chart_with_values_file(root: &Path, values_file: &str) -> String {
    render_chart_with_values_files(root, &[values_file])
}

fn render_chart_with_values_files(root: &Path, values_files: &[&str]) -> String {
    let output = Command::new("helm")
        .current_dir(root)
        .args(["template", "atlas-contract", "ops/k8s/charts/bijux-atlas"])
        .args(
            values_files
                .iter()
                .flat_map(|values_file| ["-f", *values_file])
                .collect::<Vec<_>>(),
        )
        .output()
        .expect("helm template");
    let values_label = values_files.join(", ");
    assert!(
        output.status.success(),
        "helm template must pass for {values_label}:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("helm template utf8")
}

fn render_networkpolicy_with_values_file(root: &Path, values_file: &str) -> String {
    let output = Command::new("helm")
        .current_dir(root)
        .args([
            "template",
            "atlas-contract",
            "ops/k8s/charts/bijux-atlas",
            "-f",
            values_file,
            "--show-only",
            "templates/networkpolicy.yaml",
        ])
        .output()
        .expect("helm template");
    assert!(
        output.status.success(),
        "helm template networkpolicy must pass for {values_file}:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("helm template utf8")
}

fn yaml_bool(value: &YamlValue) -> Option<bool> {
    value.as_bool()
}

fn yaml_str<'a>(value: &'a YamlValue) -> Option<&'a str> {
    value.as_str()
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
fn deployment_template_keeps_probe_resource_and_security_policies_explicit() {
    let root = repo_root();
    let text = read(&root.join("ops/k8s/charts/bijux-atlas/templates/deployment.yaml"));
    for required in [
        "readinessProbe:",
        "livenessProbe:",
        "resources:",
        "securityContext:",
        "imagePullPolicy: {{ .Values.image.pullPolicy }}",
    ] {
        assert!(
            text.contains(required),
            "deployment template missing `{required}`"
        );
    }
}

#[test]
fn immutable_release_labels_are_present_in_workload_and_service_templates() {
    let root = repo_root();
    let deployment = read(&root.join("ops/k8s/charts/bijux-atlas/templates/deployment.yaml"));
    let service = read(&root.join("ops/k8s/charts/bijux-atlas/templates/service.yaml"));
    for required in ["app.kubernetes.io/name:", "app.kubernetes.io/instance:"] {
        assert!(
            deployment.contains(required),
            "deployment template missing immutable release label `{required}`"
        );
        assert!(
            service.contains(required),
            "service template missing immutable release label `{required}`"
        );
    }
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
        deployment.contains(
            "fail \"image.repository must not include '@'; use image.digest for digests\""
        ),
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

#[test]
fn install_matrix_profiles_render_lint_and_parse_as_valid_yaml() {
    let root = repo_root();
    for values_file in install_matrix_values_files(&root) {
        let rendered = render_chart_with_values_file(&root, &values_file);
        for document in rendered
            .split("\n---\n")
            .map(str::trim)
            .filter(|doc| !doc.is_empty())
        {
            serde_yaml::from_str::<YamlValue>(document).unwrap_or_else(|err| {
                panic!("rendered YAML from {values_file} must parse cleanly: {err}")
            });
        }

        let output = Command::new("helm")
            .current_dir(&root)
            .args(["lint", "ops/k8s/charts/bijux-atlas", "-f", &values_file])
            .output()
            .expect("helm lint");
        assert!(
            output.status.success(),
            "helm lint must pass for {values_file}:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn install_matrix_profiles_keep_monitoring_scaling_and_network_policy_contracts_sound() {
    let root = repo_root();
    for values_file in install_matrix_values_files(&root) {
        let values = load_yaml(&root.join(&values_file));
        let rendered = render_chart_with_values_file(&root, &values_file);

        let metrics_enabled = values["metrics"]["enabled"].as_bool().unwrap_or(true);
        let service_monitor_enabled = values["metrics"]["serviceMonitor"]["enabled"]
            .as_bool()
            .or_else(|| values["serviceMonitor"]["enabled"].as_bool())
            .unwrap_or(true);
        if metrics_enabled && service_monitor_enabled {
            assert!(
                rendered.contains("kind: ServiceMonitor"),
                "{values_file} must render a ServiceMonitor when enabled"
            );
            assert!(
                rendered.contains("port: http"),
                "{values_file} ServiceMonitor must scrape the named http port"
            );
        }

        let hpa_enabled = values["hpa"]["enabled"].as_bool().unwrap_or(false);
        if hpa_enabled {
            assert!(
                rendered.contains("kind: HorizontalPodAutoscaler"),
                "{values_file} must render an HPA when enabled"
            );
            assert!(
                rendered.contains("kind: Deployment"),
                "{values_file} HPA must target a deployment-backed workload"
            );
        }

        let network_policy_enabled = values["networkPolicy"]["enabled"].as_bool().unwrap_or(true);
        if network_policy_enabled {
            assert!(
                rendered.contains("kind: NetworkPolicy"),
                "{values_file} must render a NetworkPolicy when enabled"
            );
            assert!(
                !rendered.contains("cidr: \"0.0.0.0/0\"") && !rendered.contains("cidr: \"::/0\""),
                "{values_file} must not render a wide-open egress CIDR"
            );
        }
    }
}

#[test]
fn networkpolicy_rendered_goldens_match_canonical_modes() {
    let root = repo_root();
    let examples = [
        (
            "ops/k8s/values/networkpolicy-internet-only.yaml",
            "ops/k8s/examples/networkpolicy/internet-only.yaml",
        ),
        (
            "ops/k8s/values/kind.yaml",
            "ops/k8s/examples/networkpolicy/cluster-aware.yaml",
        ),
        (
            "ops/k8s/values/networkpolicy-custom.yaml",
            "ops/k8s/examples/networkpolicy/custom.yaml",
        ),
    ];

    for (values_file, golden_file) in examples {
        let rendered = render_networkpolicy_with_values_file(&root, values_file);
        let expected = read(&root.join(golden_file));
        assert_eq!(
            rendered.trim(),
            expected.trim(),
            "network policy render snapshot drift for {values_file}"
        );
    }
}

#[test]
fn networkpolicy_disabled_profile_renders_no_manifest() {
    let root = repo_root();
    let rendered = render_chart_with_values_file(&root, "ops/k8s/values/dev.yaml");
    assert!(
        !rendered.contains("kind: NetworkPolicy"),
        "dev profile must not render a NetworkPolicy when disabled"
    );
}

#[test]
fn networkpolicy_profile_semantics_match_selected_modes() {
    let root = repo_root();
    let profiles = [
        "ops/k8s/values/ci.yaml",
        "ops/k8s/values/kind.yaml",
        "ops/k8s/values/prod.yaml",
        "ops/k8s/values/perf.yaml",
        "ops/k8s/values/offline.yaml",
    ];

    for values_file in profiles {
        let values = load_yaml(&root.join(values_file));
        let rendered = render_networkpolicy_with_values_file(&root, values_file);
        let network_policy = &values["networkPolicy"];

        let egress_mode = yaml_str(&network_policy["mode"])
            .or_else(|| yaml_str(&network_policy["egress"]["mode"]))
            .unwrap_or("internet-only");
        let ingress_mode = yaml_str(&network_policy["ingressMode"])
            .or_else(|| yaml_str(&network_policy["ingress"]["mode"]))
            .unwrap_or("same-namespace");
        let allow_dns = yaml_bool(&network_policy["allowDns"])
            .or_else(|| yaml_bool(&network_policy["egress"]["allowDns"]))
            .unwrap_or(false);
        let metrics_enabled = yaml_bool(&values["metrics"]["enabled"]).unwrap_or(false);
        let monitoring_namespace = yaml_str(&network_policy["monitoring"]["allowNamespace"])
            .or_else(|| yaml_str(&network_policy["allowMonitoringNamespace"]))
            .unwrap_or("");

        assert!(
            !rendered.contains("\n    - from:\n        - podSelector: {}"),
            "{values_file} must not render ingress allow-all when network policy is enabled"
        );

        if ingress_mode == "same-namespace" {
            assert!(
                rendered.contains("kubernetes.io/metadata.name: \"default\"")
                    || rendered.contains("kubernetes.io/metadata.name: default"),
                "{values_file} same-namespace mode must scope ingress by release namespace"
            );
        }

        if egress_mode == "cluster-aware" {
            assert!(
                rendered.contains("namespaceSelector:"),
                "{values_file} cluster-aware mode must render namespaceSelector-based egress"
            );
            assert!(
                rendered.contains("podSelector: {}"),
                "{values_file} cluster-aware mode must render explicit podSelector for in-cluster deps"
            );
            assert!(
                !rendered.contains("ipBlock:"),
                "{values_file} cluster-aware mode must not rely on CIDR-only egress rules"
            );
        }

        if egress_mode == "internet-only" {
            assert!(
                rendered.contains("port: 443") && rendered.contains("port: 80"),
                "{values_file} internet-only mode must restrict egress to HTTP/HTTPS"
            );
        }

        if allow_dns {
            assert!(
                rendered.contains("port: 53") && rendered.contains("protocol: UDP"),
                "{values_file} allowDns=true must render DNS egress rules"
            );
        }

        if metrics_enabled && !monitoring_namespace.is_empty() {
            assert!(
                rendered.contains(&format!("- \"{monitoring_namespace}\"")),
                "{values_file} metrics-enabled policy must allow monitoring namespace ingress"
            );
        }
    }
}

#[test]
fn kind_profile_networkpolicy_remains_reachable_shape_for_simulation() {
    let root = repo_root();
    let rendered = render_networkpolicy_with_values_file(&root, "ops/k8s/values/kind.yaml");
    assert!(
        rendered.contains("kind: NetworkPolicy"),
        "kind profile must render a NetworkPolicy for simulation coverage"
    );
    assert!(
        rendered.contains("port: 8080"),
        "kind profile NetworkPolicy must keep the service port reachable"
    );
}

#[test]
fn networkpolicy_budget_and_prod_exception_rules_are_governed() {
    let root = repo_root();
    let governed_profiles = [
        ("ops/k8s/values/prod.yaml", true),
        ("ops/k8s/values/prod-minimal.yaml", true),
        ("ops/k8s/values/prod-ha.yaml", true),
        ("ops/k8s/values/prod-airgap.yaml", true),
        ("ops/k8s/values/perf.yaml", false),
        ("ops/k8s/values/kind.yaml", false),
    ];

    for (values_file, prod_like) in governed_profiles {
        let values = load_yaml(&root.join(values_file));
        let network_policy = &values["networkPolicy"];
        let egress_mode = yaml_str(&network_policy["mode"])
            .or_else(|| yaml_str(&network_policy["egress"]["mode"]))
            .unwrap_or("internet-only");
        let cidr_budget = network_policy["cidrBudget"]["maxAllowCidrs"]
            .as_i64()
            .unwrap_or(8);
        let allow_cidrs = network_policy["egress"]["allowCidrs"]
            .as_sequence()
            .map(|items| items.len() as i64)
            .unwrap_or(0);
        let exceptions = &network_policy["exceptions"];
        let relaxed_egress = yaml_bool(&exceptions["relaxedEgress"]).unwrap_or(false);
        let prod_disable_allowed = yaml_bool(&exceptions["prodDisableAllowed"]).unwrap_or(false);
        let owner = yaml_str(&exceptions["owner"]).unwrap_or("");
        let expires_on = yaml_str(&exceptions["expiresOn"]).unwrap_or("");

        if allow_cidrs > cidr_budget {
            assert!(
                relaxed_egress,
                "{values_file} exceeds network policy CIDR budget without relaxedEgress exception"
            );
        }

        if prod_like && egress_mode == "disabled" {
            assert!(
                prod_disable_allowed,
                "{values_file} may not disable network policy without an explicit prod exception"
            );
        }

        if relaxed_egress || prod_disable_allowed {
            assert!(
                !owner.is_empty() && !expires_on.is_empty(),
                "{values_file} exceptions must declare both owner and expiry"
            );
        }
    }
}

#[test]
fn prod_profiles_keep_admin_endpoints_disabled_without_registered_exception() {
    let root = repo_root();
    let exceptions = load_json(&root.join("ops/k8s/admin-endpoints-exceptions.json"));
    let exception_profiles = exceptions["exceptions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row["profile"].as_str())
        .collect::<BTreeSet<_>>();
    for values_file in [
        "ops/k8s/values/prod.yaml",
        "ops/k8s/values/prod-minimal.yaml",
        "ops/k8s/values/prod-ha.yaml",
        "ops/k8s/values/prod-airgap.yaml",
    ] {
        let values = load_yaml(&root.join(values_file));
        let admin_enabled = values["server"]["adminEndpoints"]["enabled"]
            .as_bool()
            .unwrap_or(false);
        if admin_enabled {
            let profile_name = Path::new(values_file)
                .file_stem()
                .and_then(|value| value.to_str())
                .expect("profile stem");
            assert!(
                exception_profiles.contains(profile_name),
                "{values_file} enables admin endpoints without a registered owner/expiry exception"
            );
        }
    }
}

#[test]
fn networkpolicy_render_contains_expected_policy_shape() {
    let root = repo_root();
    let internet_only = render_networkpolicy_with_values_file(
        &root,
        "ops/k8s/values/networkpolicy-internet-only.yaml",
    );
    assert!(
        internet_only.contains("policyTypes:\n    - Ingress\n    - Egress"),
        "internet-only policy must render both policy types when ingress and egress are configured"
    );
    assert!(
        !internet_only.contains("cidr: \"0.0.0.0/0\"") && !internet_only.contains("cidr: \"::/0\""),
        "network policy must not render wide-open default-route CIDRs"
    );

    let cluster_aware = render_networkpolicy_with_values_file(&root, "ops/k8s/values/kind.yaml");
    assert!(
        cluster_aware.contains("operator: In"),
        "cluster-aware policy must use explicit namespace label membership"
    );
}
