// SPDX-License-Identifier: Apache-2.0

pub(crate) fn run_ops_k8s_plan(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (render_path, index_path) = resolve_render_inputs(&repo_root, &run_id, &profile.name)
        .map_err(|e| e.to_stable_message())?;
    let index_json: Value = serde_json::from_str(
        &fs::read_to_string(&index_path)
            .map_err(|err| format!("failed to read {}: {err}", index_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", index_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": format!("k8s plan profile={} run_id={}", profile.name, run_id.as_str()),
        "rows": [{
            "profile": profile.name,
            "run_id": run_id.as_str(),
            "render_path": render_path.display().to_string(),
            "render_index_path": index_path.display().to_string(),
            "index": index_json
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_apply(
    args: &crate::cli::OpsK8sApplyArgs,
    dry_run: bool,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !args.apply && !dry_run {
        return Err("OPS_USAGE_ERROR: k8s apply requires explicit --apply".to_string());
    }
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s apply requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("OPS_EFFECT_ERROR: k8s apply requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (render_path, _) = resolve_render_inputs(&repo_root, &run_id, &profile.name)
        .map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    if !dry_run {
        ensure_k8s_safety(&process, &repo_root, &profile, common.force, "bijux-atlas")
            .map_err(|e| e.to_stable_message())?;
    }
    let mut apply_args = vec![
        "apply".to_string(),
        "-n".to_string(),
        "bijux-atlas".to_string(),
        "-f".to_string(),
        render_path.display().to_string(),
    ];
    if dry_run {
        apply_args.push("--dry-run=client".to_string());
    }
    let (stdout, event) = process
        .run_subprocess("kubectl", &apply_args, &repo_root)
        .map_err(|e| e.to_stable_message())?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": if dry_run {"k8s dry-run completed"} else {"k8s apply completed"},
        "rows": [{
            "profile": profile.name,
            "run_id": run_id.as_str(),
            "dry_run": dry_run,
            "render_path": render_path.display().to_string(),
            "stdout": stdout,
            "subprocess_event": event
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn conformance_summary(deployments: &Value, pods: &Value) -> (Vec<String>, Vec<Value>) {
    let mut errors = Vec::new();
    let mut rows = Vec::new();
    if let Some(items) = deployments.get("items").and_then(Value::as_array) {
        for item in items {
            let name = item
                .get("metadata")
                .and_then(|v| v.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let desired = item
                .get("status")
                .and_then(|v| v.get("replicas"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let ready = item
                .get("status")
                .and_then(|v| v.get("readyReplicas"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            if ready < desired {
                errors.push(format!("deployment `{name}` ready {ready}/{desired}"));
            }
            rows.push(serde_json::json!({"kind":"deployment","name":name,"desired":desired,"ready":ready}));
        }
    }
    if let Some(items) = pods.get("items").and_then(Value::as_array) {
        for item in items {
            let name = item
                .get("metadata")
                .and_then(|v| v.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let phase = item
                .get("status")
                .and_then(|v| v.get("phase"))
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            if phase != "Running" && phase != "Succeeded" {
                errors.push(format!("pod `{name}` phase={phase}"));
            }
            rows.push(serde_json::json!({"kind":"pod","name":name,"phase":phase}));
        }
    }
    (errors, rows)
}

pub(crate) fn run_ops_k8s_conformance(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s conformance requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_k8s_safety(&process, &repo_root, &profile, common.force, "bijux-atlas")
        .map_err(|e| e.to_stable_message())?;
    let (dep_stdout, _) = process
        .run_subprocess(
            "kubectl",
            &[
                "get".to_string(),
                "deployments".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ],
            &repo_root,
        )
        .map_err(|e| e.to_stable_message())?;
    let (pod_stdout, _) = process
        .run_subprocess(
            "kubectl",
            &[
                "get".to_string(),
                "pods".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ],
            &repo_root,
        )
        .map_err(|e| e.to_stable_message())?;
    let deployments: Value = serde_json::from_str(&dep_stdout)
        .map_err(|e| format!("failed parsing deployments json: {e}"))?;
    let pods: Value =
        serde_json::from_str(&pod_stdout).map_err(|e| format!("failed parsing pods json: {e}"))?;
    let (errors, rows) = conformance_summary(&deployments, &pods);
    let error_count = errors.len();
    let payload = serde_json::json!({
        "schema_version":1,
        "text": if errors.is_empty() {"k8s conformance passed"} else {"k8s conformance failed"},
        "rows": rows,
        "errors": errors,
        "summary":{"total":1,"errors": error_count,"warnings":0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if error_count == 0 { 0 } else { 1 }))
}

pub(crate) fn run_ops_k8s_wait(args: &crate::cli::OpsK8sWaitArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s wait requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_k8s_safety(&process, &repo_root, &profile, common.force, "bijux-atlas")
        .map_err(|e| e.to_stable_message())?;
    let start = Instant::now();
    let timeout = format!("{}s", args.timeout_seconds);
    let checks = vec![
        vec![
            "wait".to_string(),
            "deployment".to_string(),
            "--all".to_string(),
            "-n".to_string(),
            "bijux-atlas".to_string(),
            "--for=condition=Available".to_string(),
            format!("--timeout={timeout}"),
        ],
        vec![
            "wait".to_string(),
            "pod".to_string(),
            "--all".to_string(),
            "-n".to_string(),
            "bijux-atlas".to_string(),
            "--for=condition=Ready".to_string(),
            format!("--timeout={timeout}"),
        ],
    ];
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for argv in checks {
        match process.run_subprocess("kubectl", &argv, &repo_root) {
            Ok((stdout, event)) => rows
                .push(serde_json::json!({"argv":argv,"stdout":stdout,"event":event,"status":"ok"})),
            Err(err) => {
                errors.push(err.to_stable_message());
                rows.push(serde_json::json!({"argv":argv,"status":"failed"}));
                if common.fail_fast {
                    break;
                }
            }
        }
    }
    let payload = serde_json::json!({
        "schema_version":1,
        "text": if errors.is_empty() {"k8s wait passed"} else {"k8s wait failed"},
        "rows": rows,
        "errors": errors,
        "summary":{"total":1,"errors": errors.len(),"warnings":0},
        "elapsed_ms": start.elapsed().as_millis()
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((
        rendered,
        if payload["errors"].as_array().is_some_and(|v| v.is_empty()) {
            0
        } else {
            1
        },
    ))
}

pub(crate) fn run_ops_k8s_logs(args: &crate::cli::OpsK8sLogsArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s logs requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_k8s_safety(&process, &repo_root, &profile, common.force, "bijux-atlas")
        .map_err(|e| e.to_stable_message())?;
    let pod = args
        .pod
        .clone()
        .unwrap_or_else(|| "deployment/bijux-atlas".to_string());
    let argv = vec![
        "logs".to_string(),
        "-n".to_string(),
        "bijux-atlas".to_string(),
        pod,
        format!("--tail={}", args.tail),
    ];
    let (stdout, event) = process
        .run_subprocess("kubectl", &argv, &repo_root)
        .map_err(|e| e.to_stable_message())?;
    let payload = serde_json::json!({"schema_version":1,"text":"k8s logs collected","rows":[{"stdout":stdout,"event":event}],"summary":{"total":1,"errors":0,"warnings":0}});
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_port_forward(
    args: &crate::cli::OpsK8sPortForwardArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s port-forward requires --allow-subprocess".to_string());
    }
    if !common.allow_network {
        return Err("OPS_EFFECT_ERROR: k8s port-forward requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_k8s_safety(&process, &repo_root, &profile, common.force, "bijux-atlas")
        .map_err(|e| e.to_stable_message())?;
    let payload = serde_json::json!({
        "schema_version":1,
        "text":"k8s port-forward command prepared",
        "rows":[{
            "resource": args.resource,
            "local_port": args.local_port,
            "remote_port": args.remote_port,
            "argv": ["kubectl","port-forward","--address","127.0.0.1",&args.resource,&format!("{}:{}", args.local_port, args.remote_port)]
        }],
        "summary":{"total":1,"errors":0,"warnings":0}
    });
    let _ = repo_root;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

