// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::access_guard::{ensure_kind_context, ensure_namespace_exists};
use crate::kubernetes::execution::{KubernetesCommandRunner, SubprocessCapture};
use crate::kubernetes::safety_policy::expected_kind_context;
use crate::lifecycle::install_status::{
    install_plan_inventory, install_render_path, load_profile_intent,
};
use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::workspace::profiles::{load_profiles, resolve_ops_root, resolve_profile};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub struct StackInstallRequest<'a> {
    pub ops_root: Option<PathBuf>,
    pub requested_profile: Option<String>,
    pub run_id: &'a str,
    pub evidence_mode: bool,
    pub plan_mode: bool,
    pub dry_run_mode: &'a str,
    pub enable_kind: bool,
    pub enable_apply: bool,
    pub allow_subprocess: bool,
    pub allow_write: bool,
    pub allow_network: bool,
    pub force: bool,
}

pub fn stack_install_payload(
    runner: &(impl SimulationCommandRunner + KubernetesCommandRunner),
    repo_root: &Path,
    request: StackInstallRequest<'_>,
) -> Result<(Value, i32), String> {
    let ops_root = resolve_ops_root(repo_root, request.ops_root).map_err(|err| err.detail())?;
    let mut profiles = load_profiles(&ops_root).map_err(|err| err.detail())?;
    profiles.sort_by(|left, right| left.name.cmp(&right.name));
    let profile =
        resolve_profile(request.requested_profile, &profiles).map_err(|err| err.detail())?;

    if !request.plan_mode && !request.allow_subprocess {
        return Err("install execution requires --allow-subprocess".to_string());
    }
    if (request.enable_apply || request.enable_kind) && !request.allow_write {
        return Err("install apply/kind requires --allow-write".to_string());
    }
    if (request.enable_apply || request.enable_kind) && !request.allow_network {
        return Err("install apply/kind requires --allow-network".to_string());
    }

    let mut steps = Vec::new();
    if request.enable_kind {
        steps.push("kind cluster ensure".to_string());
        if !request.plan_mode {
            let kind_config = repo_root.join(&profile.cluster_config);
            let kind_args = vec![
                "create".to_string(),
                "cluster".to_string(),
                "--name".to_string(),
                profile.kind_profile.clone(),
                "--config".to_string(),
                kind_config.display().to_string(),
            ];
            if let Err(error) = SimulationCommandRunner::run(runner, "kind", &kind_args, repo_root)
            {
                if !error.contains("already exists") {
                    return Err(error);
                }
            }
        }
    }

    if request.enable_apply {
        steps.push("kubectl apply".to_string());
        if !request.plan_mode {
            ensure_kind_context(runner, &profile.kind_profile, request.force)?;
            ensure_namespace_exists(runner, "bijux-atlas", request.dry_run_mode)?;
            let render_path = install_render_path(repo_root, request.run_id, &profile.name);
            let mut apply_args = vec![
                "apply".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-f".to_string(),
                render_path.display().to_string(),
            ];
            if request.dry_run_mode == "client" {
                apply_args.push("--dry-run=client".to_string());
            }
            let _: SubprocessCapture =
                KubernetesCommandRunner::run(runner, "kubectl", &apply_args, repo_root)?;
        }
    }

    if !request.enable_kind && !request.enable_apply {
        steps.push("validate-only".to_string());
    }

    let render_path = install_render_path(repo_root, request.run_id, &profile.name);
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
    let profile_intent = load_profile_intent(repo_root, &profile.name)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile.name,
        "run_id": request.run_id,
        "evidence_mode": request.evidence_mode,
        "plan_mode": request.plan_mode,
        "dry_run": request.dry_run_mode,
        "steps": steps,
        "kind_context_expected": expected_kind_context(&profile.kind_profile),
        "profile_intent": profile_intent,
        "install_plan": render_inventory,
    });

    if request.evidence_mode {
        if !request.allow_write {
            return Err("ops install --evidence requires --allow-write".to_string());
        }
        let evidence_rel = format!(
            "artifacts/ops/evidence/{}/install-evidence.json",
            request.run_id
        );
        let evidence_path = repo_root.join(&evidence_rel);
        if let Some(parent) = evidence_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let evidence_payload = serde_json::json!({
            "schema_version": 1,
            "kind": "ops_install_evidence",
            "run_id": request.run_id,
            "profile": payload["profile"],
            "dry_run": request.dry_run_mode,
            "plan_mode": request.plan_mode,
            "steps": payload["steps"],
            "install_plan": payload["install_plan"],
            "kind_context_expected": payload["kind_context_expected"],
            "profile_intent": payload["profile_intent"],
        });
        std::fs::write(
            &evidence_path,
            serde_json::to_string_pretty(&evidence_payload).map_err(|err| err.to_string())?,
        )
        .map_err(|err| format!("failed to write {}: {err}", evidence_path.display()))?;
    }

    let text = if request.plan_mode {
        format!(
            "install plan generated for profile `{}`",
            payload["profile"].as_str().unwrap_or("unknown")
        )
    } else {
        format!(
            "install completed for profile `{}`",
            payload["profile"].as_str().unwrap_or("unknown")
        )
    };
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "text": text,
            "rows": [payload],
            "summary": {"total": 1, "errors": 0, "warnings": 0}
        }),
        0,
    ))
}

#[cfg(test)]
mod tests {
    use super::{stack_install_payload, StackInstallRequest};
    use crate::kubernetes::execution::{KubernetesCommandRunner, SubprocessCapture};
    use crate::lifecycle::simulation::context::SimulationCommandRunner;
    use serde_json::Value;
    use std::path::Path;

    struct DummyRunner;

    impl SimulationCommandRunner for DummyRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<(String, Value), String> {
            unreachable!("plan-only tests should not spawn subprocesses")
        }
    }

    impl KubernetesCommandRunner for DummyRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<SubprocessCapture, String> {
            unreachable!("plan-only tests should not spawn subprocesses")
        }
    }

    #[test]
    fn stack_install_payload_reports_missing_render_path_in_plan_mode() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack/kind")).expect("mkdir kind");
        std::fs::write(
            root.path().join("ops/stack/profiles.json"),
            r#"{"profiles":[{"name":"developer","kind_profile":"atlas-kind","cluster_config":"ops/stack/kind/cluster.yaml"}]}"#,
        )
        .expect("write profiles");
        std::fs::write(
            root.path().join("ops/stack/kind/cluster.yaml"),
            "kind: Cluster",
        )
        .expect("write cluster config");

        let (payload, exit_code) = stack_install_payload(
            &DummyRunner,
            root.path(),
            StackInstallRequest {
                ops_root: None,
                requested_profile: None,
                run_id: "atlas-run",
                evidence_mode: false,
                plan_mode: true,
                dry_run_mode: "none",
                enable_kind: false,
                enable_apply: false,
                allow_subprocess: false,
                allow_write: false,
                allow_network: false,
                force: false,
            },
        )
        .expect("install payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["rows"][0]["profile"], "developer");
        assert_eq!(payload["rows"][0]["steps"][0], "validate-only");
        assert!(payload["rows"][0]["install_plan"]["missing_render_path"].is_string());
    }

    #[test]
    fn stack_install_payload_writes_owned_evidence_report() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack/kind")).expect("mkdir kind");
        std::fs::write(
            root.path().join("ops/stack/profiles.json"),
            r#"{"profiles":[{"name":"developer","kind_profile":"atlas-kind","cluster_config":"ops/stack/kind/cluster.yaml"}]}"#,
        )
        .expect("write profiles");
        std::fs::write(
            root.path().join("ops/stack/kind/cluster.yaml"),
            "kind: Cluster",
        )
        .expect("write cluster config");

        stack_install_payload(
            &DummyRunner,
            root.path(),
            StackInstallRequest {
                ops_root: None,
                requested_profile: None,
                run_id: "atlas-run",
                evidence_mode: true,
                plan_mode: true,
                dry_run_mode: "none",
                enable_kind: false,
                enable_apply: false,
                allow_subprocess: false,
                allow_write: true,
                allow_network: false,
                force: false,
            },
        )
        .expect("install payload");

        let evidence_path = root
            .path()
            .join("artifacts/ops/evidence/atlas-run/install-evidence.json");
        let evidence = std::fs::read_to_string(&evidence_path).expect("read evidence");

        assert!(evidence.contains("\"kind\": \"ops_install_evidence\""));
        assert!(evidence.contains("\"profile\": \"developer\""));
    }
}
