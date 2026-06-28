// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::path::Path;

const CHART_YAML_REL: &str = "ops/k8s/charts/bijux-atlas/Chart.yaml";
const CHART_LOCK_REL: &str = "ops/k8s/charts/bijux-atlas/Chart.lock";

pub fn build_chart_dependency_sbom_payload(
    repo_root: &Path,
    run_id: &str,
) -> Result<serde_json::Value, String> {
    let chart_yaml_path = repo_root.join(CHART_YAML_REL);
    let chart_yaml_text = std::fs::read_to_string(&chart_yaml_path)
        .map_err(|err| format!("failed to read {}: {err}", chart_yaml_path.display()))?;
    let chart_yaml: serde_yaml::Value = serde_yaml::from_str(&chart_yaml_text)
        .map_err(|err| format!("failed to parse {}: {err}", chart_yaml_path.display()))?;

    let dependencies = chart_yaml
        .as_mapping()
        .and_then(|map| map.get(serde_yaml::Value::String("dependencies".to_string())))
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for dep in dependencies {
        let Some(dep_map) = dep.as_mapping() else {
            errors.push("Chart.yaml dependencies entries must be objects".to_string());
            continue;
        };
        let name = dep_map
            .get(serde_yaml::Value::String("name".to_string()))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let version = dep_map
            .get(serde_yaml::Value::String("version".to_string()))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let repository = dep_map
            .get(serde_yaml::Value::String("repository".to_string()))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if version.contains('^')
            || version.contains('~')
            || version.contains('>')
            || version.contains('<')
            || version.contains('*')
            || version.contains('x')
        {
            errors.push(format!(
                "dependency `{name}` must pin an exact version, found `{version}`"
            ));
        }
        rows.push(serde_json::json!({
            "name": name,
            "version": version,
            "repository": repository
        }));
    }
    rows.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));

    let chart_lock_path = repo_root.join(CHART_LOCK_REL);
    let lock_exists = chart_lock_path.is_file();
    if !rows.is_empty() && !lock_exists {
        errors.push(format!(
            "Chart.lock is required when Chart.yaml declares dependencies: {}",
            chart_lock_path.display()
        ));
    }
    if lock_exists {
        let lock_text = std::fs::read_to_string(&chart_lock_path)
            .map_err(|err| format!("failed to read {}: {err}", chart_lock_path.display()))?;
        let lock_yaml: serde_yaml::Value = serde_yaml::from_str(&lock_text)
            .map_err(|err| format!("failed to parse {}: {err}", chart_lock_path.display()))?;
        let lock_rows = lock_yaml
            .as_mapping()
            .and_then(|map| map.get(serde_yaml::Value::String("dependencies".to_string())))
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        let mut lock_set = BTreeSet::new();
        for dep in lock_rows {
            let Some(dep_map) = dep.as_mapping() else {
                continue;
            };
            let name = dep_map
                .get(serde_yaml::Value::String("name".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let version = dep_map
                .get(serde_yaml::Value::String("version".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default()
                .to_string();
            lock_set.insert((name, version));
        }
        let mut chart_set = BTreeSet::new();
        for row in &rows {
            chart_set.insert((
                row["name"].as_str().unwrap_or_default().to_string(),
                row["version"].as_str().unwrap_or_default().to_string(),
            ));
        }
        if chart_set != lock_set {
            errors.push(
                "Chart.lock dependencies must match Chart.yaml dependency name/version pairs"
                    .to_string(),
            );
        }
    }

    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "ops_chart_dependency_sbom",
        "run_id": run_id,
        "chart": "ops/k8s/charts/bijux-atlas",
        "dependencies": rows,
        "lock_file": {
            "path": CHART_LOCK_REL,
            "exists": lock_exists
        },
        "summary": {
            "total": rows.len(),
            "errors": errors.len(),
            "warnings": 0
        },
        "errors": errors
    }))
}

#[cfg(test)]
mod tests {
    use super::build_chart_dependency_sbom_payload;

    #[test]
    fn chart_dependency_sbom_reads_owned_chart_contracts() {
        let root = tempfile::tempdir().expect("tempdir");
        let chart_root = root.path().join("ops/k8s/charts/bijux-atlas");
        std::fs::create_dir_all(&chart_root).expect("mkdir chart");
        std::fs::write(
            chart_root.join("Chart.yaml"),
            r#"
apiVersion: v2
name: bijux-atlas
dependencies:
  - name: redis
    version: "18.17.0"
    repository: "https://charts.bitnami.com/bitnami"
"#,
        )
        .expect("write chart");
        std::fs::write(
            chart_root.join("Chart.lock"),
            r#"
dependencies:
  - name: redis
    version: "18.17.0"
"#,
        )
        .expect("write lock");

        let payload =
            build_chart_dependency_sbom_payload(root.path(), "owned-run").expect("sbom payload");

        assert_eq!(payload["chart"], "ops/k8s/charts/bijux-atlas");
        assert_eq!(payload["dependencies"].as_array().expect("deps").len(), 1);
        assert_eq!(payload["summary"]["errors"], 0);
    }
}
