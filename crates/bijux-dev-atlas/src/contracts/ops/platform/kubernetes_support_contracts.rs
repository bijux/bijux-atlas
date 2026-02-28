fn test_ops_k8s_001_chart_renders_static(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-001";
    let test_id = "ops.k8s.chart_renders_static";
    let chart_root = ctx.repo_root.join("ops/k8s/charts/bijux-atlas");
    let required = [
        "ops/k8s/charts/bijux-atlas/Chart.yaml",
        "ops/k8s/charts/bijux-atlas/values.yaml",
        "ops/k8s/charts/bijux-atlas/values.schema.json",
    ];
    let mut violations = Vec::new();
    for rel in required {
        if !ctx.repo_root.join(rel).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "required chart source is missing",
                Some(rel.to_string()),
            ));
        }
    }
    let templates_dir = chart_root.join("templates");
    let mut template_count = 0usize;
    if let Ok(entries) = std::fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path
                .extension()
                .and_then(|v| v.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("tpl"))
            {
                template_count += 1;
            }
        }
    }
    if template_count == 0 {
        violations.push(violation(
            contract_id,
            test_id,
            "helm chart must include at least one template file",
            Some("ops/k8s/charts/bijux-atlas/templates".to_string()),
        ));
    }

    let Some(chart) = read_yaml_value(&ctx.repo_root.join("ops/k8s/charts/bijux-atlas/Chart.yaml")) else {
        violations.push(violation(
            contract_id,
            test_id,
            "Chart.yaml must be valid yaml",
            Some("ops/k8s/charts/bijux-atlas/Chart.yaml".to_string()),
        ));
        return TestResult::Fail(violations);
    };
    let chart_name = chart.get("name").and_then(|v| v.as_str()).unwrap_or("");
    if chart_name != "bijux-atlas" {
        violations.push(violation(
            contract_id,
            test_id,
            "Chart.yaml name must be bijux-atlas",
            Some("ops/k8s/charts/bijux-atlas/Chart.yaml".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn read_yaml_value(path: &Path) -> Option<serde_yaml::Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&text).ok()
}
