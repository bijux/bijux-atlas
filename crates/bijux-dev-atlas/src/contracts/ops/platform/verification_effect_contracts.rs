// SPDX-License-Identifier: Apache-2.0

fn effect_violation(contract_id: &str, test_id: &str, message: &str, file: &str) -> Violation {
    violation(contract_id, test_id, message, Some(file.to_string()))
}

fn test_ops_stack_005_kind_profiles_consistent(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-005";
    let test_id = "ops.stack.kind_profiles_consistent";
    let profiles_rel = "ops/stack/profiles.json";
    let Some(profiles) = read_json(&ctx.repo_root.join(profiles_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack profiles manifest must be parseable",
            Some(profiles_rel.to_string()),
        )]);
    };
    let Some(rows) = profiles.get("profiles").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack profiles manifest must include profiles array",
            Some(profiles_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut required = BTreeSet::from([
        "ops/stack/kind/cluster-dev.yaml".to_string(),
        "ops/stack/kind/cluster-perf.yaml".to_string(),
        "ops/stack/kind/cluster-small.yaml".to_string(),
    ]);
    for item in rows {
        let cluster = item
            .get("cluster_config")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if cluster.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "profile entry must include cluster_config",
                Some(profiles_rel.to_string()),
            ));
            continue;
        }
        if !ctx.repo_root.join(&cluster).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "cluster_config file referenced by profile must exist",
                Some(cluster.clone()),
            ));
            continue;
        }
        required.remove(&cluster);
    }
    for cluster in &required {
        violations.push(violation(
            contract_id,
            test_id,
            "dev, perf, and small kind cluster configs must be referenced by profiles",
            Some(cluster.to_string()),
        ));
    }
    for rel in [
        "ops/stack/kind/cluster-dev.yaml",
        "ops/stack/kind/cluster-perf.yaml",
        "ops/stack/kind/cluster-small.yaml",
    ] {
        let Some(doc) = read_yaml_value(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "kind cluster config must be parseable yaml",
                Some(rel.to_string()),
            ));
            continue;
        };
        let has_kind = doc.get("kind").and_then(|v| v.as_str()) == Some("Cluster");
        let has_api = doc.get("apiVersion").and_then(|v| v.as_str()).is_some();
        let has_nodes = doc
            .get("nodes")
            .and_then(|v| v.as_sequence())
            .is_some_and(|nodes| !nodes.is_empty());
        if !has_kind || !has_api || !has_nodes {
            violations.push(violation(
                contract_id,
                test_id,
                "kind cluster config must include kind/apiVersion/nodes",
                Some(rel.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_006_ports_inventory_matches_stack(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-006";
    let test_id = "ops.stack.ports_inventory_matches_stack";
    let sample_rel = "ops/stack/tests/goldens/stack-ports-inventory.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack ports inventory sample must be parseable",
            Some(sample_rel.to_string()),
        )]);
    };
    let Some(ports) = sample.as_object() else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack ports inventory sample must be a json object",
            Some(sample_rel.to_string()),
        )]);
    };
    let graph_rel = "ops/stack/generated/dependency-graph.json";
    let Some(graph) = read_json(&ctx.repo_root.join(graph_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack dependency graph must be parseable",
            Some(graph_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut seen_endpoints = BTreeSet::new();
    for (service, endpoint) in ports {
        let Some(endpoint) = endpoint.as_str() else {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory entries must map to endpoint strings",
                Some(sample_rel.to_string()),
            ));
            continue;
        };
        if !endpoint.starts_with("http://127.0.0.1:") {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory endpoint must use loopback host and explicit port",
                Some(sample_rel.to_string()),
            ));
            continue;
        }
        if !seen_endpoints.insert(endpoint.to_string()) {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory must not contain duplicate endpoints",
                Some(sample_rel.to_string()),
            ));
        }
        if service.trim().is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory service key must not be empty",
                Some(sample_rel.to_string()),
            ));
        }
    }
    let component_text = graph
        .get("profiles")
        .and_then(|v| v.as_object())
        .map(|profiles| format!("{profiles:?}"))
        .unwrap_or_default();
    for service in ["atlas", "prometheus"] {
        if !ports.contains_key(service) {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory must include required stack services",
                Some(sample_rel.to_string()),
            ));
        }
    }
    if !component_text.contains("grafana.yaml") && ports.contains_key("grafana") {
        violations.push(violation(
            contract_id,
            test_id,
            "grafana endpoint cannot be declared when grafana component is absent from dependency graph",
            Some(graph_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_007_health_report_generator_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-007";
    let test_id = "ops.stack.health_report_generator_contract";
    let sample_rel = "ops/stack/tests/goldens/stack-health-report.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack health report sample must be parseable",
            Some(sample_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if sample
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .is_none_or(|version| version < 1)
    {
        violations.push(violation(
            contract_id,
            test_id,
            "stack health report sample must include schema_version >= 1",
            Some(sample_rel.to_string()),
        ));
    }
    if sample
        .get("checks")
        .and_then(|v| v.as_object())
        .is_none_or(|checks| checks.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "stack health report sample must include non-empty checks object",
            Some(sample_rel.to_string()),
        ));
    }
    let index_rel = "ops/stack/generated/stack-index.json";
    let Some(index) = read_json(&ctx.repo_root.join(index_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack index must be parseable",
            Some(index_rel.to_string()),
        )]);
    };
    if index
        .get("generated_by")
        .and_then(|v| v.as_str())
        .is_none_or(|v| !v.contains("ops generate"))
    {
        violations.push(violation(
            contract_id,
            test_id,
            "stack index must declare ops generator provenance",
            Some(index_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_008_stack_commands_registered(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-008";
    let test_id = "ops.stack.stack_commands_registered";
    let rel = "ops/_generated/control-plane-surface-list.json";
    let Some(surface) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "control-plane surface snapshot must be parseable",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut stack_verbs = BTreeSet::new();
    if let Some(entries) = surface
        .get("ops_taxonomy")
        .and_then(|v| v.get("entries"))
        .and_then(|v| v.as_array())
    {
        for entry in entries {
            if entry.get("domain").and_then(|v| v.as_str()) == Some("stack") {
                if let Some(verb) = entry.get("verb").and_then(|v| v.as_str()) {
                    stack_verbs.insert(verb.to_string());
                }
            }
        }
    }
    for required in ["up", "down"] {
        if !stack_verbs.contains(required) {
            violations.push(violation(
                contract_id,
                test_id,
                "stack command surface must include up/down verbs",
                Some(rel.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_009_offline_profile_policy(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-009";
    let test_id = "ops.stack.offline_profile_policy";
    let evolution_rel = "ops/stack/evolution-policy.json";
    let Some(policy) = read_json(&ctx.repo_root.join(evolution_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack evolution policy must be parseable",
            Some(evolution_rel.to_string()),
        )]);
    };
    let profiles_rel = "ops/stack/profiles.json";
    let Some(profiles) = read_json(&ctx.repo_root.join(profiles_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack profiles manifest must be parseable",
            Some(profiles_rel.to_string()),
        )]);
    };
    let profile_names: BTreeSet<String> = profiles
        .get("profiles")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("name").and_then(|v| v.as_str()))
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let compatibility_profiles: BTreeSet<String> = policy
        .get("compatibility")
        .and_then(|v| v.get("cluster_profiles"))
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.as_str())
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let claims_offline = compatibility_profiles.contains("offline")
        || compatibility_profiles.contains("airgap")
        || profile_names.contains("offline")
        || profile_names.contains("airgap");
    if claims_offline && !profile_names.contains("offline") && !profile_names.contains("airgap") {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "offline/airgap compatibility claims require an offline or airgap stack profile",
            Some(profiles_rel.to_string()),
        )]);
    }
    TestResult::Pass
}

fn test_ops_stack_e_001_kind_cluster_up_profile_dev(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-001";
    let test_id = "ops.stack.effect.kind_cluster_up_profile_dev";
    let profile = ctx.repo_root.join("ops/stack/kind/cluster-dev.yaml");
    if !profile.exists() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "kind dev profile config must exist for effect execution",
            "ops/stack/kind/cluster-dev.yaml",
        )]);
    }
    TestResult::Pass
}

fn test_ops_stack_e_002_core_components_present(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-002";
    let test_id = "ops.stack.effect.core_components_present";
    let required = [
        "ops/stack/minio/minio.yaml",
        "ops/stack/redis/redis.yaml",
        "ops/stack/prometheus/prometheus.yaml",
        "ops/stack/otel/otel-collector.yaml",
        "ops/stack/grafana/grafana.yaml",
    ];
    let mut violations = Vec::new();
    for rel in required {
        if !ctx.repo_root.join(rel).exists() {
            violations.push(effect_violation(
                contract_id,
                test_id,
                "effect component manifest is missing",
                rel,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_e_003_ports_inventory_mapped(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-003";
    let test_id = "ops.stack.effect.ports_inventory_mapped";
    let sample_rel = "ops/stack/tests/goldens/stack-ports-inventory.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack ports inventory sample must be parseable",
            sample_rel,
        )]);
    };
    let has_endpoints = sample
        .as_object()
        .is_some_and(|obj| !obj.is_empty() && obj.values().all(|v| v.as_str().is_some()));
    if !has_endpoints {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack ports inventory sample must map service names to endpoint strings",
            sample_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_stack_e_004_stack_health_report_generated(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-004";
    let test_id = "ops.stack.effect.health_report_generated";
    let sample_rel = "ops/stack/tests/goldens/stack-health-report.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack health report sample must be parseable",
            sample_rel,
        )]);
    };
    if sample
        .get("checks")
        .and_then(|v| v.as_object())
        .is_none_or(|checks| checks.is_empty())
    {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack health report sample must include non-empty checks object",
            sample_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_k8s_e_001_chart_defaults_rendered(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-001";
    let test_id = "ops.k8s.effect.chart_defaults_rendered";
    let chart = "ops/k8s/charts/bijux-atlas";
    let values = "ops/k8s/charts/bijux-atlas/values.yaml";
    let output = match run_ops_effect_command(
        ctx,
        OpsEffectCommand {
            contract_id,
            test_id,
            program: "helm",
            args: &[
                "template",
                "bijux-atlas",
                chart,
                "--namespace",
                "bijux-atlas",
                "-f",
                values,
            ],
            stdout_rel: "helm/defaults.stdout.log".to_string(),
            stderr_rel: "helm/defaults.stderr.log".to_string(),
            network_allowed: false,
        },
    ) {
        Ok(output) => output,
        Err(result) => return result,
    };
    let rendered = String::from_utf8_lossy(&output.stdout);
    if let Some(root) = ops_effect_artifact_dir(ctx) {
        let path = root.join("helm/rendered.yaml");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, rendered.as_bytes());
    }
    if output.status.success() && rendered.contains("kind: Deployment") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "helm template with chart defaults must succeed and include a Deployment",
            chart,
        )])
    }
}

fn test_ops_k8s_e_002_chart_minimal_values_rendered(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-002";
    let test_id = "ops.k8s.effect.chart_minimal_values_rendered";
    let chart = "ops/k8s/charts/bijux-atlas";
    let values = "ops/k8s/values/local.yaml";
    let output = match run_ops_effect_command(
        ctx,
        OpsEffectCommand {
            contract_id,
            test_id,
            program: "helm",
            args: &[
                "template",
                "bijux-atlas",
                chart,
                "--namespace",
                "bijux-atlas",
                "-f",
                values,
            ],
            stdout_rel: "helm/minimal.stdout.log".to_string(),
            stderr_rel: "helm/minimal.stderr.log".to_string(),
            network_allowed: false,
        },
    ) {
        Ok(output) => output,
        Err(result) => return result,
    };
    let rendered = String::from_utf8_lossy(&output.stdout);
    if output.status.success() && rendered.contains("kind: Service") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "helm template with minimal values must succeed and include a Service",
            values,
        )])
    }
}

fn test_ops_k8s_e_003_kubeconform_render_validation(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-003";
    let test_id = "ops.k8s.effect.kubeconform_render_validation";
    let rendered_path = ctx
        .artifacts_root
        .as_ref()
        .map(|root| root.join("contracts/ops/helm/rendered.manifest.yaml"))
        .unwrap_or_else(|| ctx.repo_root.join("artifacts/contracts/ops/helm/rendered.manifest.yaml"));
    if let Some(parent) = rendered_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let render = match run_ops_effect_command(
        ctx,
        OpsEffectCommand {
            contract_id,
            test_id,
            program: "helm",
            args: &[
                "template",
                "bijux-atlas",
                "ops/k8s/charts/bijux-atlas",
                "--namespace",
                "bijux-atlas",
                "-f",
                "ops/k8s/charts/bijux-atlas/values.yaml",
            ],
            stdout_rel: "helm/kubeconform-render.stdout.log".to_string(),
            stderr_rel: "helm/kubeconform-render.stderr.log".to_string(),
            network_allowed: false,
        },
    ) {
        Ok(output) => output,
        Err(result) => return result,
    };
    if !render.status.success() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "helm render must succeed before kubeconform validation",
            "ops/k8s/charts/bijux-atlas",
        )]);
    }
    let _ = std::fs::write(&rendered_path, &render.stdout);
    let rendered_arg = rendered_path.display().to_string();
    let output = match run_ops_effect_command(
        ctx,
        OpsEffectCommand {
            contract_id,
            test_id,
            program: "kubeconform",
            args: &["-strict", "-summary", &rendered_arg],
            stdout_rel: "helm/kubeconform.stdout.log".to_string(),
            stderr_rel: "helm/kubeconform.stderr.log".to_string(),
            network_allowed: false,
        },
    ) {
        Ok(output) => output,
        Err(result) => return result,
    };
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let _ = write_ops_effect_json(
        ctx,
        "helm/kubeconform.json",
        &serde_json::json!({
            "schema_version": 1,
            "status": if output.status.success() { "ok" } else { "failed" },
            "stdout": stdout,
            "stderr": stderr,
        }),
    );
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "kubeconform validation failed for rendered chart manifests",
            "artifacts/contracts/ops/helm/kubeconform.json",
        )])
    }
}

fn test_ops_k8s_e_004_helm_install_contract_defined(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-004";
    let test_id = "ops.k8s.effect.helm_install_contract_defined";
    let matrix_rel = "ops/k8s/install-matrix.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "install-matrix must be parseable for effect execution",
            matrix_rel,
        )]);
    };
    let has_kind = matrix
        .get("profiles")
        .and_then(|v| v.as_array())
        .is_some_and(|rows| {
            rows.iter()
                .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                .any(|name| name == "kind")
        });
    if !has_kind {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "k8s install-matrix must include kind profile",
            matrix_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_k8s_e_005_rollout_safety_contract_satisfied(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-005";
    let test_id = "ops.k8s.effect.rollout_safety_contract_satisfied";
    let rel = "ops/k8s/rollout-safety-contract.json";
    let Some(contract) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "rollout safety contract must be parseable",
            rel,
        )]);
    };
    if contract
        .get("profiles")
        .and_then(|v| v.as_array())
        .is_none_or(|rows| rows.is_empty())
    {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "rollout safety contract must include non-empty profiles array",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_k8s_e_006_tool_versions_recorded(ctx: &RunContext) -> TestResult {
    verify_declared_tool_versions(ctx, &["helm", "kubeconform"])
}

fn test_ops_stack_e_005_kind_install_smoke(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-005";
    let test_id = "ops.stack.effect.kind_install_smoke";
    let output = match run_ops_effect_command(
        ctx,
        OpsEffectCommand {
            contract_id,
            test_id,
            program: "kind",
            args: &["get", "clusters"],
            stdout_rel: "kind/clusters.stdout.log".to_string(),
            stderr_rel: "kind/clusters.stderr.log".to_string(),
            network_allowed: false,
        },
    ) {
        Ok(output) => output,
        Err(result) => return result,
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "kind cluster discovery must succeed in the release lane environment",
            "ops/stack/kind/cluster-dev.yaml",
        )])
    }
}

fn test_ops_obs_e_001_scrape_metrics_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-E-001";
    let test_id = "ops.observe.effect.scrape_metrics_contract";
    let rel = "ops/observe/contracts/metrics.golden.prom";
    let Ok(text) = std::fs::read_to_string(ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "metrics golden contract file must be readable",
            rel,
        )]);
    };
    if text.trim().is_empty() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "metrics golden contract must not be empty",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_obs_e_002_trace_structure_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-E-002";
    let test_id = "ops.observe.effect.trace_structure_contract";
    let rel = "ops/observe/contracts/trace-structure.golden.json";
    let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "trace structure golden must be parseable",
            rel,
        )]);
    };
    if !value.is_object() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "trace structure golden must be a json object",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_obs_e_003_alerts_load_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-E-003";
    let test_id = "ops.observe.effect.alerts_load_contract";
    let files = [
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/alerts/slo-burn-rules.yaml",
    ];
    let mut violations = Vec::new();
    for rel in files {
        if read_yaml_value(&ctx.repo_root.join(rel)).is_none() {
            violations.push(effect_violation(
                contract_id,
                test_id,
                "alert rule yaml must be parseable for effect checks",
                rel,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
