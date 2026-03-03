// SPDX-License-Identifier: Apache-2.0

use std::io::Read;
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

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

fn simulation_cluster_name() -> &'static str {
    "bijux-atlas-sim"
}

fn simulation_cluster_context() -> String {
    format!("kind-{}", simulation_cluster_name())
}

fn simulation_cluster_config(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join("ops/k8s/kind/cluster.yaml")
}

fn simulation_report_path(
    repo_root: &std::path::Path,
    run_id: &RunId,
    file_name: &str,
) -> Result<std::path::PathBuf, String> {
    let path = repo_root
        .join("artifacts/ops")
        .join(run_id.as_str())
        .join("reports")
        .join(file_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

fn write_simulation_report(
    repo_root: &std::path::Path,
    run_id: &RunId,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<std::path::PathBuf, String> {
    let path = simulation_report_path(repo_root, run_id, file_name)?;
    std::fs::write(
        &path,
        serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(path)
}

fn update_simulation_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
    namespace: &str,
    install_report_path: Option<&std::path::Path>,
    install_status: Option<&str>,
    smoke_report_path: Option<&std::path::Path>,
    smoke_status: Option<&str>,
    cleanup_report_path: Option<&std::path::Path>,
    cleanup_status: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    let summary_path = simulation_report_path(repo_root, run_id, "ops-simulation-summary.json")?;
    let mut payload = if summary_path.exists() {
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&summary_path)
                .map_err(|err| format!("failed to read {}: {err}", summary_path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", summary_path.display()))?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "cluster": "kind",
            "profiles": []
        })
    };
    if !payload["profiles"].is_array() {
        payload["profiles"] = serde_json::json!([]);
    }
    let rows = payload["profiles"]
        .as_array_mut()
        .ok_or_else(|| "ops-simulation-summary.json profiles must be an array".to_string())?;
    if let Some(existing) = rows
        .iter_mut()
        .find(|row| row.get("profile").and_then(|v| v.as_str()) == Some(profile))
    {
        existing["namespace"] = serde_json::json!(namespace);
        if let Some(path) = install_report_path {
            existing["install_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = install_status {
            existing["install_status"] = serde_json::json!(status);
        }
        if let Some(path) = smoke_report_path {
            existing["smoke_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = smoke_status {
            existing["smoke_status"] = serde_json::json!(status);
        }
        if let Some(path) = cleanup_report_path {
            existing["cleanup_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = cleanup_status {
            existing["cleanup_status"] = serde_json::json!(status);
        }
    } else {
        let mut row = serde_json::json!({
            "profile": profile,
            "namespace": namespace
        });
        if let Some(path) = install_report_path {
            row["install_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = install_status {
            row["install_status"] = serde_json::json!(status);
        }
        if let Some(path) = smoke_report_path {
            row["smoke_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = smoke_status {
            row["smoke_status"] = serde_json::json!(status);
        }
        if let Some(path) = cleanup_report_path {
            row["cleanup_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = cleanup_status {
            row["cleanup_status"] = serde_json::json!(status);
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| {
        a.get("profile")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("profile").and_then(serde_json::Value::as_str))
    });
    std::fs::write(
        &summary_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", summary_path.display()))?;
    Ok(summary_path)
}

fn ensure_simulation_context(process: &OpsProcess, force: bool) -> Result<(), String> {
    let args = vec!["config".to_string(), "current-context".to_string()];
    let (stdout, _) = process
        .run_subprocess("kubectl", &args, std::path::Path::new("."))
        .map_err(|err| err.to_stable_message())?;
    let current = stdout.trim();
    let expected = simulation_cluster_context();
    if current == expected || force {
        Ok(())
    } else {
        Err(format!(
            "kubectl context guard failed: expected `{expected}` got `{current}`; pass --force to override"
        ))
    }
}

fn resolve_profile_values_file(
    repo_root: &std::path::Path,
    profile: &str,
) -> Result<std::path::PathBuf, String> {
    let path = repo_root.join("ops/k8s/values").join(format!("{profile}.yaml"));
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "missing values file {}; expected profile values at ops/k8s/values/{profile}.yaml",
            path.display()
        ))
    }
}

fn simulation_namespace(profile: &str, override_namespace: Option<&str>) -> String {
    override_namespace
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("bijux-atlas-{profile}"))
}

fn debug_artifact_path(
    repo_root: &std::path::Path,
    run_id: &RunId,
    namespace: &str,
    file_name: &str,
) -> Result<std::path::PathBuf, String> {
    let path = repo_root
        .join("artifacts/ops")
        .join(run_id.as_str())
        .join("debug")
        .join(namespace)
        .join(file_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

fn write_debug_artifact(
    repo_root: &std::path::Path,
    run_id: &RunId,
    namespace: &str,
    file_name: &str,
    content: &str,
) -> Result<std::path::PathBuf, String> {
    let path = debug_artifact_path(repo_root, run_id, namespace, file_name)?;
    std::fs::write(&path, content)
        .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(path)
}

fn load_profile_registry(
    repo_root: &std::path::Path,
    profile: &str,
) -> Result<Option<serde_json::Value>, String> {
    let path = repo_root.join("ops/k8s/values/profiles.json");
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
        .and_then(|rows| rows.as_array())
        .and_then(|rows| {
            rows.iter()
                .find(|row| row.get("id").and_then(|v| v.as_str()) == Some(profile))
                .cloned()
        }))
}

fn extract_configmap_env_keys(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
) -> Result<Vec<String>, String> {
    let render_path = install_render_path(repo_root, run_id.as_str(), profile);
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
        if value
            .get("kind")
            .and_then(serde_yaml::Value::as_str)
            != Some("ConfigMap")
        {
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

fn record_kubeconform_result(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
) -> serde_json::Value {
    let render_path = install_render_path(repo_root, run_id.as_str(), profile);
    let args = vec![
        "-summary".to_string(),
        render_path.display().to_string(),
    ];
    match process.run_subprocess("kubeconform", &args, repo_root) {
        Ok((stdout, event)) => serde_json::json!({
            "status": "ok",
            "stdout": stdout,
            "event": event,
            "render_path": render_path.display().to_string()
        }),
        Err(err) => serde_json::json!({
            "status": "failed",
            "error": err.to_stable_message(),
            "render_path": render_path.display().to_string()
        }),
    }
}

fn runtime_allowlist_status(repo_root: &std::path::Path) -> serde_json::Value {
    let path = repo_root.join("configs/contracts/env.schema.json");
    serde_json::json!({
        "status": if path.exists() { "ok" } else { "failed" },
        "path": path.display().to_string()
    })
}

fn emit_debug_bundle_report(
    repo_root: &std::path::Path,
    run_id: &RunId,
    namespace: &str,
    category: &str,
    files: &[std::path::PathBuf],
) -> Result<std::path::PathBuf, String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "category": category,
        "status": "ok",
        "files": files.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()
    });
    write_simulation_report(
        repo_root,
        run_id,
        &format!("ops-debug-bundle-{category}.json"),
        &payload,
    )
}

fn run_simulation_wait(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
    timeout_seconds: u64,
) -> (Vec<serde_json::Value>, Vec<String>, u128) {
    let start = Instant::now();
    let timeout = format!("{timeout_seconds}s");
    let checks = vec![
        vec![
            "wait".to_string(),
            "deployment/bijux-atlas".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--for=condition=Available".to_string(),
            format!("--timeout={timeout}"),
        ],
        vec![
            "wait".to_string(),
            "pod".to_string(),
            "--all".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--for=condition=Ready".to_string(),
            format!("--timeout={timeout}"),
        ],
    ];
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for argv in checks {
        match process.run_subprocess("kubectl", &argv, repo_root) {
            Ok((stdout, event)) => rows.push(serde_json::json!({
                "argv": argv,
                "stdout": stdout,
                "event": event,
                "status": "ok"
            })),
            Err(err) => {
                errors.push(err.to_stable_message());
                rows.push(serde_json::json!({
                    "argv": argv,
                    "status": "failed"
                }));
            }
        }
    }
    (rows, errors, start.elapsed().as_millis())
}

fn wait_for_local_port(port: u16, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!("timed out waiting for localhost:{port}"))
}

fn perform_http_check(local_port: u16, path: &str) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let mut stream =
        TcpStream::connect(("127.0.0.1", local_port)).map_err(|err| format!("connect failed: {err}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| format!("set read timeout failed: {err}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| format!("set write timeout failed: {err}"))?;
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write failed: {err}"))?;
    let _ = stream.shutdown(Shutdown::Write);
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|err| format!("read failed: {err}"))?;
    let response_text = String::from_utf8_lossy(&response);
    let mut lines = response_text.lines();
    let status_line = lines.next().unwrap_or_default().to_string();
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let body = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|offset| &response[offset + 4..])
        .unwrap_or_default();
    Ok(serde_json::json!({
        "path": path,
        "status": status_code,
        "latency_ms": started.elapsed().as_millis(),
        "body_sha256": sha256_hex(&String::from_utf8_lossy(body))
    }))
}

fn run_smoke_checks(
    repo_root: &std::path::Path,
    namespace: &str,
    local_port: u16,
) -> Result<Vec<serde_json::Value>, String> {
    let mut child = std::process::Command::new("kubectl")
        .args([
            "port-forward",
            "-n",
            namespace,
            "--address",
            "127.0.0.1",
            "service/bijux-atlas",
            &format!("{local_port}:8080"),
        ])
        .current_dir(repo_root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|err| format!("failed to start kubectl port-forward: {err}"))?;
    let checks = (|| -> Result<Vec<serde_json::Value>, String> {
        wait_for_local_port(local_port, Duration::from_secs(10))?;
        let mut rows = Vec::new();
        for path in ["/healthz", "/readyz", "/v1/version"] {
            rows.push(perform_http_check(local_port, path)?);
        }
        Ok(rows)
    })();
    let _ = child.kill();
    let _ = child.wait();
    checks
}

pub(crate) fn run_ops_kind_up(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind up requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind up requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let config_path = simulation_cluster_config(&repo_root);
    let args = vec![
        "create".to_string(),
        "cluster".to_string(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
        "--config".to_string(),
        config_path.display().to_string(),
    ];
    let result = process.run_subprocess("kind", &args, &repo_root);
    let (status, detail) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => {
            let stable = err.to_stable_message();
            if stable.contains("already exists") {
                ("ok", serde_json::json!({"detail": "cluster already exists"}))
            } else {
                ("failed", serde_json::json!({"error": stable}))
            }
        }
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "up",
        "status": status,
        "details": {
            "cluster_name": simulation_cluster_name(),
            "cluster_config": config_path.display().to_string(),
            "context": simulation_cluster_context(),
            "result": detail
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind cluster ready" } else { "kind cluster failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "up",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_kind_down(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind down requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind down requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let args = vec![
        "delete".to_string(),
        "cluster".to_string(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
    ];
    let result = process.run_subprocess("kind", &args, &repo_root);
    let (status, detail) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => ("failed", serde_json::json!({"error": err.to_stable_message()})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "down",
        "status": status,
        "details": {
            "cluster_name": simulation_cluster_name(),
            "result": detail
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind cluster deleted" } else { "kind cluster delete failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "down",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_kind_status(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind status requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind status requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let args = vec![
        "--context".to_string(),
        simulation_cluster_context(),
        "get".to_string(),
        "nodes".to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let result = process.run_subprocess("kubectl", &args, &repo_root);
    let (status, details) = match result {
        Ok((stdout, event)) => {
            let json: serde_json::Value = serde_json::from_str(&stdout)
                .map_err(|err| format!("failed to parse kubectl nodes json: {err}"))?;
            let rows = json
                .get("items")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|item| {
                    let name = item["metadata"]["name"].as_str().unwrap_or("unknown");
                    let ready = item["status"]["conditions"]
                        .as_array()
                        .is_some_and(|conditions| {
                            conditions.iter().any(|condition| {
                                condition["type"].as_str() == Some("Ready")
                                    && condition["status"].as_str() == Some("True")
                            })
                        });
                    serde_json::json!({"name": name, "ready": ready})
                })
                .collect::<Vec<_>>();
            ("ok", serde_json::json!({"event": event, "nodes": rows}))
        }
        Err(err) => ("failed", serde_json::json!({"error": err.to_stable_message()})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "status",
        "status": status,
        "details": details
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind cluster status collected" } else { "kind cluster status failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "status",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_kind_preload(
    args: &crate::cli::OpsKindPreloadArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("kind preload-image requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind preload-image requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let argv = vec![
        "load".to_string(),
        "docker-image".to_string(),
        args.image.clone(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
    ];
    let result = process.run_subprocess("kind", &argv, &repo_root);
    let (status, details) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => ("failed", serde_json::json!({"error": err.to_stable_message()})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "preload-image",
        "status": status,
        "details": {
            "image": args.image,
            "result": details
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind image preload complete" } else { "kind image preload failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "preload-image",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_helm_install(
    args: &crate::cli::OpsHelmReleaseArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm install requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm install requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm install requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    let values_file = resolve_profile_values_file(&repo_root, &profile)?;
    let chart_path = repo_root.join("ops/k8s/charts/bijux-atlas");
    let helm_args = vec![
        "upgrade".to_string(),
        "--install".to_string(),
        "bijux-atlas".to_string(),
        chart_path.display().to_string(),
        "--namespace".to_string(),
        namespace.clone(),
        "--create-namespace".to_string(),
        "--values".to_string(),
        values_file.display().to_string(),
    ];
    let (helm_stdout, helm_event) = process
        .run_subprocess("helm", &helm_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let (wait_rows, wait_errors, wait_ms) =
        run_simulation_wait(&process, &repo_root, &namespace, args.timeout_seconds);
    let smoke_rows = if wait_errors.is_empty() {
        run_smoke_checks(&repo_root, &namespace, 18080)?
    } else {
        Vec::new()
    };
    let smoke_errors = smoke_rows
        .iter()
        .filter(|row| row["status"].as_u64().unwrap_or(0) != 200)
        .map(|row| {
            format!(
                "{} returned status {}",
                row["path"].as_str().unwrap_or("unknown"),
                row["status"].as_u64().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>();
    let errors = wait_errors
        .iter()
        .cloned()
        .chain(smoke_errors.iter().cloned())
        .collect::<Vec<_>>();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let smoke_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": if wait_errors.is_empty() && smoke_errors.is_empty() { "ok" } else { "failed" },
        "checks": smoke_rows
    });
    let smoke_report_path =
        write_simulation_report(&repo_root, &run_id, "ops-smoke.json", &smoke_payload)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "details": {
            "helm": {
                "stdout": helm_stdout,
                "event": helm_event,
                "values_file": values_file.display().to_string(),
                "chart_path": chart_path.display().to_string()
            },
            "readiness_wait": {
                "elapsed_ms": wait_ms,
                "rows": wait_rows,
                "errors": wait_errors
            },
            "kubeconform": record_kubeconform_result(&process, &repo_root, &run_id, &profile),
            "configmap_env_keys": extract_configmap_env_keys(&repo_root, &run_id, &profile)?,
            "runtime_allowlist": runtime_allowlist_status(&repo_root),
            "smoke": {
                "report_path": smoke_report_path.display().to_string(),
                "checks": smoke_payload["checks"].clone()
            },
            "profile_intent": load_profile_intent(&repo_root, &profile)?,
            "profile_metadata": load_profile_registry(&repo_root, &profile)?
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-install.json", &payload)?;
    let summary_path = update_simulation_summary(
        &repo_root,
        &run_id,
        &profile,
        &namespace,
        Some(&report_path),
        Some(status),
        Some(&smoke_report_path),
        Some(smoke_payload["status"].as_str().unwrap_or("failed")),
        None,
        None,
    )?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "helm install completed" } else { "helm install failed" },
        "rows": [{
            "schema_version": 1,
            "profile": payload["profile"].clone(),
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "report_path": report_path.display().to_string(),
            "summary_report_path": summary_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if errors.is_empty() { 0 } else { 1 }))
}

pub(crate) fn run_ops_helm_uninstall(
    args: &crate::cli::OpsHelmReleaseArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm uninstall requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm uninstall requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm uninstall requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    let helm_args = vec![
        "uninstall".to_string(),
        "bijux-atlas".to_string(),
        "--namespace".to_string(),
        namespace.clone(),
    ];
    let (helm_stdout, helm_event) = process
        .run_subprocess("helm", &helm_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let cleanup_args = vec![
        "get".to_string(),
        "all".to_string(),
        "-n".to_string(),
        namespace.clone(),
        "-o".to_string(),
        "name".to_string(),
    ];
    let (cleanup_stdout, cleanup_event) = process
        .run_subprocess("kubectl", &cleanup_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let leftovers = cleanup_stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let status = if leftovers.is_empty() { "ok" } else { "failed" };
    let cleanup_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "leftovers": leftovers
    });
    let cleanup_report_path =
        write_simulation_report(&repo_root, &run_id, "ops-cleanup.json", &cleanup_payload)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile,
        "cluster": "kind",
        "namespace": cleanup_payload["namespace"].clone(),
        "status": status,
        "details": {
            "helm": {
                "stdout": helm_stdout,
                "event": helm_event
            },
            "cleanup_check": {
                "report_path": cleanup_report_path.display().to_string(),
                "leftovers": cleanup_payload["leftovers"].clone(),
                "event": cleanup_event
            }
        }
    });
    let report_path =
        write_simulation_report(&repo_root, &run_id, "ops-uninstall.json", &payload)?;
    let summary_path = update_simulation_summary(
        &repo_root,
        &run_id,
        &profile,
        &namespace,
        None,
        None,
        None,
        None,
        Some(&cleanup_report_path),
        Some(status),
    )?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "helm uninstall completed" } else { "helm uninstall left resources" },
        "rows": [{
            "schema_version": 1,
            "profile": payload["profile"].clone(),
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "report_path": report_path.display().to_string(),
            "summary_report_path": summary_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_smoke(args: &crate::cli::OpsSmokeArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("smoke requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("smoke requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("smoke requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    let checks = run_smoke_checks(&repo_root, &namespace, args.local_port)?;
    let errors = checks
        .iter()
        .filter(|row| row["status"].as_u64().unwrap_or(0) != 200)
        .map(|row| {
            format!(
                "{} returned status {}",
                row["path"].as_str().unwrap_or("unknown"),
                row["status"].as_u64().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "checks": checks
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-smoke.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "smoke checks passed" } else { "smoke checks failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "checks": payload["checks"].clone(),
            "report_path": report_path.display().to_string()
        }],
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if errors.is_empty() { 0 } else { 1 }))
}

fn run_collect_command(
    args: &crate::cli::OpsCollectArgs,
    category: &str,
    file_name: &str,
    argv: Vec<String>,
) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err(format!("{category} collect requires --allow-subprocess"));
    }
    if !common.allow_write {
        return Err(format!("{category} collect requires --allow-write"));
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    let (stdout, event) = process
        .run_subprocess("kubectl", &argv, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let artifact_path = write_debug_artifact(&repo_root, &run_id, &namespace, file_name, &stdout)?;
    let report_path = emit_debug_bundle_report(
        &repo_root,
        &run_id,
        &namespace,
        category,
        std::slice::from_ref(&artifact_path),
    )?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": format!("{category} collected"),
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "namespace": namespace,
            "category": category,
            "status": "ok",
            "files": [artifact_path.display().to_string()],
            "report_path": report_path.display().to_string(),
            "event": event
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_logs_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "logs",
        "pod-logs.txt",
        vec![
            "logs".to_string(),
            "-n".to_string(),
            namespace,
            "deployment/bijux-atlas".to_string(),
            "--tail=500".to_string(),
        ],
    )
}

pub(crate) fn run_ops_describe_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "describe",
        "describe.txt",
        vec![
            "describe".to_string(),
            "-n".to_string(),
            namespace,
            "deployment/bijux-atlas".to_string(),
            "service/bijux-atlas".to_string(),
        ],
    )
}

pub(crate) fn run_ops_events_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "events",
        "events.txt",
        vec![
            "get".to_string(),
            "events".to_string(),
            "-n".to_string(),
            namespace,
            "--sort-by=.metadata.creationTimestamp".to_string(),
        ],
    )
}

pub(crate) fn run_ops_resources_snapshot(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "resources",
        "resources.yaml",
        vec![
            "get".to_string(),
            "all".to_string(),
            "-n".to_string(),
            namespace,
            "-o".to_string(),
            "yaml".to_string(),
        ],
    )
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
