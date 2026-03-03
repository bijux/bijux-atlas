// SPDX-License-Identifier: Apache-2.0

fn install_render_path(repo_root: &std::path::Path, run_id: &str, profile: &str) -> std::path::PathBuf {
    repo_root
        .join("artifacts/ops")
        .join(run_id)
        .join(format!("render/{profile}/helm/render.yaml"))
}

fn install_plan_inventory(rendered_manifest: &str) -> serde_json::Value {
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
        if matches!(kind.as_str(), "Role" | "RoleBinding" | "ClusterRole" | "ClusterRoleBinding" | "ServiceAccount") {
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

fn load_profile_intent(
    repo_root: &std::path::Path,
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

pub(crate) fn run_ops_install(args: &cli::OpsInstallArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    if !args.plan && !common.allow_subprocess {
        return Err(OpsCommandError::Effect(
            "install execution requires --allow-subprocess".to_string(),
        )
        .to_stable_message());
    }
    if (args.apply || args.kind) && !common.allow_write {
        return Err(OpsCommandError::Effect(
            "install apply/kind requires --allow-write".to_string(),
        )
        .to_stable_message());
    }
    if (args.apply || args.kind) && !common.allow_network {
        return Err(OpsCommandError::Effect(
            "install apply/kind requires --allow-network".to_string(),
        )
        .to_stable_message());
    }

    let mut steps = Vec::new();
    let process = OpsProcess::new(common.allow_subprocess);
    if args.kind {
        steps.push("kind cluster ensure".to_string());
        if !args.plan {
            let kind_config = repo_root.join(&profile.cluster_config);
            let kind_args = vec![
                "create".to_string(),
                "cluster".to_string(),
                "--name".to_string(),
                profile.kind_profile.clone(),
                "--config".to_string(),
                kind_config.display().to_string(),
            ];
            if let Err(err) = process.run_subprocess("kind", &kind_args, &repo_root) {
                let stable = err.to_stable_message();
                if !stable.contains("already exists") {
                    return Err(stable);
                }
            }
        }
    }
    if args.apply {
        steps.push("kubectl apply".to_string());
        if !args.plan {
            ensure_kind_context(&process, &profile, common.force)
                .map_err(|e| e.to_stable_message())?;
            ensure_namespace_exists(&process, "bijux-atlas", &args.dry_run)
                .map_err(|e| e.to_stable_message())?;
            let render_path = repo_root
                .join("artifacts/ops")
                .join(run_id.as_str())
                .join(format!("render/{}/helm/render.yaml", profile.name));
            let mut apply_args = vec![
                "apply".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-f".to_string(),
                render_path.display().to_string(),
            ];
            if args.dry_run == "client" {
                apply_args.push("--dry-run=client".to_string());
            }
            let _ = process
                .run_subprocess("kubectl", &apply_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
        }
    }
    if !args.kind && !args.apply {
        steps.push("validate-only".to_string());
    }
    let render_path = install_render_path(&repo_root, run_id.as_str(), &profile.name);
    let render_inventory = if render_path.exists() {
        let rendered_manifest = std::fs::read_to_string(&render_path)
            .map_err(|err| format!("failed to read {}: {err}", render_path.display()))?;
        install_plan_inventory(&rendered_manifest)
    } else {
        serde_json::json!({
            "resources": [],
            "resource_kinds": {},
            "namespaces": [],
            "namespace_isolated": true,
            "has_crds": false,
            "has_rbac": false,
            "forbidden_objects": [],
            "missing_render_path": render_path.display().to_string(),
        })
    };
    let profile_intent = load_profile_intent(&repo_root, &profile.name)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile.name,
        "run_id": run_id.as_str(),
        "plan_mode": args.plan,
        "dry_run": args.dry_run,
        "steps": steps,
        "kind_context_expected": expected_kind_context(&profile),
        "profile_intent": profile_intent,
        "install_plan": render_inventory,
    });
    let text = if args.plan {
        format!("install plan generated for profile `{}`", profile.name)
    } else {
        format!("install completed for profile `{}`", profile.name)
    };
    let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

#[cfg(test)]
mod install_status_tests {
    use super::{install_plan_inventory, install_render_path, load_profile_intent};

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
