// SPDX-License-Identifier: Apache-2.0
//! Status commands and local tests for install-status flows.

use super::*;

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod install_status_tests {
    use super::{
        contains_common_secret_pattern, install_plan_inventory, install_render_path,
        load_profile_intent, redact_sensitive_text,
    };

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
                .unwrap_or_else(|| panic!("namespaces"))
                .len(),
            1
        );
        assert_eq!(payload["has_rbac"].as_bool(), Some(false));
        assert_eq!(payload["has_crds"].as_bool(), Some(false));
        assert_eq!(payload["namespace_isolated"].as_bool(), Some(true));
        assert!(payload["forbidden_objects"]
            .as_array()
            .is_some_and(|rows| rows.is_empty()));
        assert_eq!(
            payload["resource_kinds"]["Deployment"].as_u64(),
            Some(1)
        );
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
            .unwrap_or_else(|| panic!("forbidden"));
        assert!(forbidden.iter().any(|row| row.as_str().is_some_and(|value| value.contains("ClusterRole"))));
        assert!(forbidden.iter().any(|row| row.as_str().is_some_and(|value| value.contains("NodePort"))));
        assert_eq!(payload["has_rbac"].as_bool(), Some(true));
        assert_eq!(payload["namespace_isolated"].as_bool(), Some(true));
    }

    #[test]
    fn install_render_path_is_stable() {
        let repo_root = std::path::Path::new("/repo");
        let path = install_render_path(repo_root, "ops_run", "kind");
        assert_eq!(
            path,
            std::path::PathBuf::from("/repo/artifacts/ops/ops_run/render/kind/helm/render.yaml")
        );
    }

    #[test]
    fn load_profile_intent_returns_selected_profile() {
        let root = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        std::fs::create_dir_all(root.path().join("ops/stack"))
            .unwrap_or_else(|err| panic!("mkdir: {err}"));
        std::fs::write(
            root.path().join("ops/stack/profile-intent.json"),
            r#"{"schema_version":1,"profiles":[{"name":"ci","intended_usage":"ci","allowed_effects":["subprocess"],"required_dependencies":["kind-cluster"]}]}"#,
        )
        .unwrap_or_else(|err| panic!("intent: {err}"));
        let intent = load_profile_intent(root.path(), "ci")
            .unwrap_or_else(|err| panic!("load: {err}"))
            .unwrap_or_else(|| panic!("profile"));
        assert_eq!(intent["name"].as_str(), Some("ci"));
    }

    #[test]
    fn redact_sensitive_text_removes_common_secret_values() {
        let source = "PASSWORD=hunter2\nTOKEN=abc123\nAuthorization: Bearer long-token\n";
        let redacted = redact_sensitive_text(source);
        assert!(!redacted.contains("hunter2"));
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("long-token"));
        assert!(redacted.contains("PASSWORD=[REDACTED]"));
        assert!(redacted.contains("TOKEN=[REDACTED]"));
    }

    #[test]
    fn contains_common_secret_pattern_detects_unredacted_markers() {
        assert!(contains_common_secret_pattern("api_key=abc"));
        assert!(contains_common_secret_pattern("authorization: bearer secret"));
        assert!(!contains_common_secret_pattern("api_key=[REDACTED]"));
    }
}

pub(crate) fn run_ops_status(args: &cli::OpsStatusArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(common.allow_subprocess);
    let (payload, text) = match args.target {
        OpsStatusTarget::Local => {
            let toolchain_path = ops_root.join("inventory/toolchain.json");
            let toolchain = std::fs::read_to_string(&toolchain_path).map_err(|err| {
                OpsCommandError::Manifest(format!(
                    "failed to read {}: {err}",
                    toolchain_path.display()
                ))
                .to_stable_message()
            })?;
            let toolchain_json: serde_json::Value =
                serde_json::from_str(&toolchain).map_err(|err| {
                    OpsCommandError::Schema(format!(
                        "failed to parse {}: {err}",
                        toolchain_path.display()
                    ))
                    .to_stable_message()
                })?;
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "local",
                    "repo_root": repo_root.display().to_string(),
                    "ops_root": ops_root.display().to_string(),
                    "profile": profile,
                    "toolchain": toolchain_json,
                }),
                format!(
                    "ops status local: profile={} repo_root={} ops_root={}",
                    profile.name,
                    repo_root.display(),
                    ops_root.display(),
                ),
            )
        }
        OpsStatusTarget::K8s => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "status k8s requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let kubectl_args = vec![
                "get".to_string(),
                "all".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ];
            let (stdout, _) = process
                .run_subprocess("kubectl", &kubectl_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let value: serde_json::Value = serde_json::from_str(&stdout).map_err(|err| {
                OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                    .to_stable_message()
            })?;
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "k8s",
                    "profile": profile.name,
                    "resources": value
                }),
                "ops status k8s collected".to_string(),
            )
        }
        OpsStatusTarget::Pods => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "status pods requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let kubectl_args = vec![
                "get".to_string(),
                "pods".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ];
            let (stdout, _) = process
                .run_subprocess("kubectl", &kubectl_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let value: serde_json::Value = serde_json::from_str(&stdout).map_err(|err| {
                OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                    .to_stable_message()
            })?;
            let mut pods = value
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            pods.sort_by(|a, b| {
                a.get("metadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|v| v.as_str())
                    .cmp(
                        &b.get("metadata")
                            .and_then(|m| m.get("name"))
                            .and_then(|v| v.as_str()),
                    )
            });
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "pods",
                    "profile": profile.name,
                    "pods": pods
                }),
                "ops status pods collected".to_string(),
            )
        }
        OpsStatusTarget::Endpoints => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "status endpoints requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let kubectl_args = vec![
                "get".to_string(),
                "endpoints".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ];
            let (stdout, _) = process
                .run_subprocess("kubectl", &kubectl_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let value: serde_json::Value = serde_json::from_str(&stdout).map_err(|err| {
                OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                    .to_stable_message()
            })?;
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "endpoints",
                    "profile": profile.name,
                    "resources": value
                }),
                "ops status endpoints collected".to_string(),
            )
        }
    };
    let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}
