// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn install_render_path(repo_root: &Path, run_id: &str, profile: &str) -> PathBuf {
    repo_root
        .join("artifacts/ops")
        .join(run_id)
        .join(format!("render/{profile}/helm/render.yaml"))
}

pub fn install_plan_inventory(rendered_manifest: &str) -> serde_json::Value {
    let mut resources = Vec::<serde_json::Value>::new();
    let mut namespaces = std::collections::BTreeSet::<String>::new();
    let mut kinds = std::collections::BTreeMap::<String, u64>::new();
    let mut forbidden = Vec::<String>::new();
    let mut has_rbac = false;
    let mut has_crds = false;

    for document in serde_yaml::Deserializer::from_str(rendered_manifest) {
        let value: serde_yaml::Value = match serde::Deserialize::deserialize(document) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let kind = value
            .get("kind")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if kind.is_empty() {
            continue;
        }
        let metadata = value.get("metadata");
        let name = metadata
            .and_then(|meta| meta.get("name"))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let namespace = metadata
            .and_then(|meta| meta.get("namespace"))
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string);
        if let Some(namespace) = &namespace {
            namespaces.insert(namespace.clone());
        }
        *kinds.entry(kind.clone()).or_insert(0) += 1;
        if matches!(
            kind.as_str(),
            "Role" | "RoleBinding" | "ClusterRole" | "ClusterRoleBinding" | "ServiceAccount"
        ) {
            has_rbac = true;
        }
        if kind == "CustomResourceDefinition" {
            has_crds = true;
        }
        if matches!(kind.as_str(), "ClusterRole" | "ClusterRoleBinding") {
            forbidden.push(format!("forbidden cluster-scoped RBAC object `{kind}`"));
        }
        if kind == "Service" {
            let service_type = value
                .get("spec")
                .and_then(|spec| spec.get("type"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default();
            if service_type == "NodePort" {
                forbidden.push("forbidden service type `NodePort`".to_string());
            }
        }
        resources.push(serde_json::json!({
            "kind": kind,
            "name": name,
            "namespace": namespace,
        }));
    }

    resources.sort_by(|a, b| {
        a.get("kind")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("kind").and_then(serde_json::Value::as_str))
            .then_with(|| {
                a.get("namespace")
                    .and_then(serde_json::Value::as_str)
                    .cmp(&b.get("namespace").and_then(serde_json::Value::as_str))
            })
            .then_with(|| {
                a.get("name")
                    .and_then(serde_json::Value::as_str)
                    .cmp(&b.get("name").and_then(serde_json::Value::as_str))
            })
    });
    forbidden.sort();
    forbidden.dedup();

    let namespace_isolated = namespaces
        .iter()
        .all(|namespace| namespace == "bijux-atlas");
    serde_json::json!({
        "resources": resources,
        "resource_kinds": kinds,
        "namespaces": namespaces.into_iter().collect::<Vec<_>>(),
        "namespace_isolated": namespace_isolated,
        "has_crds": has_crds,
        "has_rbac": has_rbac,
        "forbidden_objects": forbidden,
    })
}

pub fn load_profile_intent(
    repo_root: &Path,
    profile: &str,
) -> Result<Option<serde_json::Value>, String> {
    let path = repo_root.join("ops/stack/profile-intent.json");
    if !path.exists() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(value
        .get("profiles")
        .and_then(|v| v.as_array())
        .and_then(|rows| {
            rows.iter()
                .find(|row| row.get("name").and_then(|v| v.as_str()) == Some(profile))
                .cloned()
        }))
}

pub fn extract_configmap_env_keys(
    repo_root: &Path,
    run_id: &str,
    profile: &str,
) -> Result<Vec<String>, String> {
    let render_path = install_render_path(repo_root, run_id, profile);
    if !render_path.exists() {
        return Ok(Vec::new());
    }
    let rendered = std::fs::read_to_string(&render_path)
        .map_err(|err| format!("failed to read {}: {err}", render_path.display()))?;
    let mut keys = std::collections::BTreeSet::<String>::new();
    for document in serde_yaml::Deserializer::from_str(&rendered) {
        let value: serde_yaml::Value = match serde::Deserialize::deserialize(document) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("kind").and_then(serde_yaml::Value::as_str) != Some("ConfigMap") {
            continue;
        }
        let data = match value.get("data").and_then(serde_yaml::Value::as_mapping) {
            Some(data) => data,
            None => continue,
        };
        for key in data.keys().filter_map(serde_yaml::Value::as_str) {
            if key
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                keys.insert(key.to_string());
            }
        }
    }
    Ok(keys.into_iter().collect())
}

#[must_use]
pub fn runtime_env_allowlist_status(repo_root: &Path) -> serde_json::Value {
    let path = repo_root.join("configs/schemas/contracts/env.schema.json");
    serde_json::json!({
        "status": if path.exists() { "ok" } else { "failed" },
        "path": path.display().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::{
        extract_configmap_env_keys, install_plan_inventory, install_render_path,
        load_profile_intent, runtime_env_allowlist_status,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn install_plan_inventory_summarizes_resources_deterministically() {
        let manifest = r#"
apiVersion: v1
kind: Namespace
metadata:
  name: bijux-atlas
---
apiVersion: v1
kind: Service
metadata:
  name: atlas
  namespace: bijux-atlas
spec:
  type: ClusterIP
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: atlas
  namespace: bijux-atlas
"#;
        let payload = install_plan_inventory(manifest);
        assert_eq!(
            payload["namespaces"]
                .as_array()
                .expect("namespaces array")
                .len(),
            1
        );
        assert_eq!(payload["has_rbac"].as_bool(), Some(false));
        assert_eq!(payload["has_crds"].as_bool(), Some(false));
        assert_eq!(payload["namespace_isolated"].as_bool(), Some(true));
        assert!(payload["forbidden_objects"]
            .as_array()
            .is_some_and(|rows| rows.is_empty()));
        assert_eq!(payload["resource_kinds"]["Deployment"].as_u64(), Some(1));
    }

    #[test]
    fn install_plan_inventory_flags_forbidden_objects() {
        let manifest = r#"
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: atlas-admin
---
apiVersion: v1
kind: Service
metadata:
  name: atlas
spec:
  type: NodePort
"#;
        let payload = install_plan_inventory(manifest);
        let forbidden = payload["forbidden_objects"]
            .as_array()
            .expect("forbidden array");
        assert!(forbidden.iter().any(|row| row
            .as_str()
            .is_some_and(|value| value.contains("ClusterRole"))));
        assert!(forbidden
            .iter()
            .any(|row| row.as_str().is_some_and(|value| value.contains("NodePort"))));
        assert_eq!(payload["has_rbac"].as_bool(), Some(true));
        assert_eq!(payload["namespace_isolated"].as_bool(), Some(true));
    }

    #[test]
    fn install_render_path_is_stable() {
        let repo_root = Path::new("/repo");
        let path = install_render_path(repo_root, "ops_run", "kind");
        assert_eq!(
            path,
            PathBuf::from("/repo/artifacts/ops/ops_run/render/kind/helm/render.yaml")
        );
    }

    #[test]
    fn load_profile_intent_returns_selected_profile() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/stack/profile-intent.json"),
            r#"{"schema_version":1,"profiles":[{"name":"ci","intended_usage":"ci","allowed_effects":["subprocess"],"required_dependencies":["kind-cluster"]}]}"#,
        )
        .expect("write profile intent");
        let intent = load_profile_intent(root.path(), "ci")
            .expect("load profile intent")
            .expect("profile entry");
        assert_eq!(intent["name"].as_str(), Some("ci"));
    }

    #[test]
    fn extract_configmap_env_keys_reads_rendered_manifest_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        let render_path = install_render_path(root.path(), "atlas-run", "kind");
        std::fs::create_dir_all(render_path.parent().expect("render parent"))
            .expect("mkdir render parent");
        std::fs::write(
            &render_path,
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: atlas-env
data:
  LOG_LEVEL: info
  FEATURE_FLAG_X: enabled
  lower_case: ignored
"#,
        )
        .expect("write rendered manifest");

        let keys =
            extract_configmap_env_keys(root.path(), "atlas-run", "kind").expect("extract keys");

        assert_eq!(keys, vec!["FEATURE_FLAG_X", "LOG_LEVEL"]);
    }

    #[test]
    fn runtime_env_allowlist_status_reports_owned_contract_path() {
        let root = tempfile::tempdir().expect("tempdir");
        let contract_path = root.path().join("configs/schemas/contracts");
        let expected_path = root
            .path()
            .join("configs/schemas/contracts/env.schema.json")
            .display()
            .to_string();
        std::fs::create_dir_all(&contract_path).expect("mkdir contracts");
        std::fs::write(contract_path.join("env.schema.json"), "{}").expect("write env schema");

        let payload = runtime_env_allowlist_status(root.path());

        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["path"].as_str(), Some(expected_path.as_str()));
    }
}
