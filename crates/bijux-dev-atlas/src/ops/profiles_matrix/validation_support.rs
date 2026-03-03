// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) fn validate_values_file(
    validator: &serde_json::Value,
    merged_values: &serde_json::Value,
) -> Result<Vec<String>, String> {
    Ok(schema_violations(validator, merged_values, "$"))
}

pub(super) fn dataset_validation_status(
    repo_root: &Path,
    manifest_path: &Path,
    manifest_ids: &BTreeMap<String, String>,
    merged_values: &serde_json::Value,
) -> StatusReport {
    let pinned_datasets = merged_values
        .get("cache")
        .and_then(|value| value.as_object())
        .and_then(|value| value.get("pinnedDatasets"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::new();
    for entry in pinned_datasets {
        let Some(dataset_id) = entry.as_str() else {
            errors.push("pinned dataset ids must be strings".to_string());
            continue;
        };
        if !manifest_ids.contains_key(dataset_id) {
            errors.push(format!(
                "pinned dataset id `{dataset_id}` is not declared in {}",
                manifest_path.display()
            ));
        }
    }
    let status = if errors.is_empty() { "pass" } else { "fail" };
    StatusReport {
        status: status.to_string(),
        note: if errors.is_empty() {
            "pinned datasets subset of dataset manifest".to_string()
        } else {
            "pinned dataset id validation failed".to_string()
        },
        errors,
        event: ToolInvocationReport {
            binary: "ops/datasets/manifest.json".to_string(),
            args: vec![manifest_path.display().to_string()],
            cwd: repo_root.display().to_string(),
            status: status.to_string(),
            stderr: String::new(),
        },
    }
}

pub(super) fn template_profile(
    repo_root: &Path,
    helm_binary: &str,
    chart_dir: &Path,
    values_path: &Path,
    profile_name: &str,
    timeout_seconds: u64,
) -> StatusReport {
    let args = vec![
        "template".to_string(),
        format!("atlas-{profile_name}"),
        chart_dir.display().to_string(),
        "--namespace".to_string(),
        "bijux-atlas".to_string(),
        "-f".to_string(),
        values_path.display().to_string(),
    ];
    let event_base = ToolInvocationReport {
        binary: helm_binary.to_string(),
        args: args.clone(),
        cwd: repo_root.display().to_string(),
        status: "fail".to_string(),
        stderr: String::new(),
    };
    match run_with_timeout(helm_binary, &args, repo_root, timeout_seconds) {
        Ok(output) if output.status.success() => StatusReport {
            status: "pass".to_string(),
            note: String::new(),
            errors: Vec::new(),
            event: ToolInvocationReport {
                status: "pass".to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ..event_base
            },
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            StatusReport {
                status: "fail".to_string(),
                note: "helm guard failure".to_string(),
                errors: vec![stderr.clone()],
                event: ToolInvocationReport {
                    stderr,
                    ..event_base
                },
            }
        }
        Err(message) => StatusReport {
            status: "fail".to_string(),
            note: "helm invocation failure".to_string(),
            errors: vec![message.clone()],
            event: ToolInvocationReport {
                stderr: message,
                ..event_base
            },
        },
    }
}

pub(super) fn template_profile_output(
    repo_root: &Path,
    helm_binary: &str,
    chart_dir: &Path,
    values_path: &Path,
    profile_name: &str,
    timeout_seconds: u64,
) -> Result<(String, String), String> {
    let args = vec![
        "template".to_string(),
        format!("atlas-{profile_name}"),
        chart_dir.display().to_string(),
        "--namespace".to_string(),
        "bijux-atlas".to_string(),
        "-f".to_string(),
        values_path.display().to_string(),
    ];
    let output = run_with_timeout(helm_binary, &args, repo_root, timeout_seconds)?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).trim().to_string(),
    ))
}

pub(super) fn kubeconform_profile(
    repo_root: &Path,
    rendered_yaml: &str,
    timeout_seconds: u64,
    run_kubeconform: bool,
) -> StatusReport {
    let args = vec![
        "-strict".to_string(),
        "-summary".to_string(),
        "-ignore-missing-schemas".to_string(),
        "<rendered-manifest>".to_string(),
    ];
    let event_base = ToolInvocationReport {
        binary: "kubeconform".to_string(),
        args,
        cwd: repo_root.display().to_string(),
        status: "skipped".to_string(),
        stderr: String::new(),
    };
    if !run_kubeconform {
        return StatusReport {
            status: "skipped".to_string(),
            note: "kubeconform disabled".to_string(),
            errors: Vec::new(),
            event: event_base,
        };
    }
    if !binary_exists("kubeconform") {
        return StatusReport {
            status: "skipped".to_string(),
            note: "kubeconform missing in PATH".to_string(),
            errors: Vec::new(),
            event: event_base,
        };
    }
    let temp_dir = repo_root.join("artifacts/ops/profile-render-matrix/tmp");
    if let Err(err) = std::fs::create_dir_all(&temp_dir) {
        return StatusReport {
            status: "fail".to_string(),
            note: "kubeconform staging failure".to_string(),
            errors: vec![format!("failed to create {}: {err}", temp_dir.display())],
            event: ToolInvocationReport {
                status: "fail".to_string(),
                stderr: format!("failed to create {}: {err}", temp_dir.display()),
                ..event_base
            },
        };
    }
    let rendered_path = temp_dir.join("rendered.yaml");
    if let Err(err) = std::fs::write(&rendered_path, rendered_yaml) {
        return StatusReport {
            status: "fail".to_string(),
            note: "kubeconform staging failure".to_string(),
            errors: vec![format!(
                "failed to write {}: {err}",
                rendered_path.display()
            )],
            event: ToolInvocationReport {
                status: "fail".to_string(),
                stderr: format!("failed to write {}: {err}", rendered_path.display()),
                ..event_base
            },
        };
    }
    let exec_args = vec![
        "-strict".to_string(),
        "-summary".to_string(),
        "-ignore-missing-schemas".to_string(),
        rendered_path.display().to_string(),
    ];
    match run_with_timeout("kubeconform", &exec_args, repo_root, timeout_seconds) {
        Ok(output) if output.status.success() => StatusReport {
            status: "pass".to_string(),
            note: "kubeconform validated rendered resources".to_string(),
            errors: Vec::new(),
            event: ToolInvocationReport {
                status: "pass".to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ..event_base
            },
        },
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = if stdout.is_empty() {
                stderr.clone()
            } else {
                stdout.clone()
            };
            StatusReport {
                status: "fail".to_string(),
                note: "kubeconform resource failure".to_string(),
                errors: vec![message.clone()],
                event: ToolInvocationReport {
                    status: "fail".to_string(),
                    stderr: if stderr.is_empty() { message } else { stderr },
                    ..event_base
                },
            }
        }
        Err(message) => StatusReport {
            status: "fail".to_string(),
            note: "kubeconform invocation failure".to_string(),
            errors: vec![message.clone()],
            event: ToolInvocationReport {
                status: "fail".to_string(),
                stderr: message,
                ..event_base
            },
        },
    }
}

fn matches_schema(schema: &serde_json::Value, instance: &serde_json::Value) -> bool {
    schema_violations(schema, instance, "$").is_empty()
}

fn schema_violations(
    schema: &serde_json::Value,
    instance: &serde_json::Value,
    instance_path: &str,
) -> Vec<String> {
    let Some(schema_obj) = schema.as_object() else {
        return Vec::new();
    };
    let mut violations = Vec::new();

    if let Some(const_value) = schema_obj.get("const") {
        if instance != const_value {
            violations.push(format!("{instance_path}: value must equal {const_value}"));
        }
    }

    if let Some(enum_values) = schema_obj.get("enum").and_then(|value| value.as_array()) {
        if !enum_values.iter().any(|value| value == instance) {
            violations.push(format!("{instance_path}: value is not in enum"));
        }
    }

    if let Some(type_name) = schema_obj.get("type").and_then(|value| value.as_str()) {
        let type_matches = match type_name {
            "object" => instance.is_object(),
            "array" => instance.is_array(),
            "string" => instance.is_string(),
            "integer" => instance.as_i64().is_some() || instance.as_u64().is_some(),
            "number" => instance.is_number(),
            "boolean" => instance.is_boolean(),
            "null" => instance.is_null(),
            _ => true,
        };
        if !type_matches {
            violations.push(format!("{instance_path}: value must be {type_name}"));
            return violations;
        }
    }

    if let Some(pattern) = schema_obj.get("pattern").and_then(|value| value.as_str()) {
        if let Some(text) = instance.as_str() {
            match Regex::new(pattern) {
                Ok(regex) => {
                    if !regex.is_match(text) {
                        violations.push(format!("{instance_path}: string does not match pattern"));
                    }
                }
                Err(err) => violations.push(format!(
                    "{instance_path}: invalid schema pattern `{pattern}`: {err}"
                )),
            }
        }
    }

    if let Some(min_length) = schema_obj.get("minLength").and_then(|value| value.as_u64()) {
        if let Some(text) = instance.as_str() {
            if text.chars().count() < min_length as usize {
                violations.push(format!("{instance_path}: string shorter than minLength"));
            }
        }
    }

    if let Some(max_length) = schema_obj.get("maxLength").and_then(|value| value.as_u64()) {
        if let Some(text) = instance.as_str() {
            if text.chars().count() > max_length as usize {
                violations.push(format!("{instance_path}: string longer than maxLength"));
            }
        }
    }

    if let Some(minimum) = schema_obj.get("minimum").and_then(|value| value.as_f64()) {
        if let Some(number) = instance.as_f64() {
            if number < minimum {
                violations.push(format!("{instance_path}: number below minimum"));
            }
        }
    }

    if let Some(required) = schema_obj.get("required").and_then(|value| value.as_array()) {
        if let Some(obj) = instance.as_object() {
            for field in required.iter().filter_map(|value| value.as_str()) {
                if !obj.contains_key(field) {
                    violations.push(format!("{instance_path}: missing required key `{field}`"));
                }
            }
        }
    }

    if let Some(properties) = schema_obj.get("properties").and_then(|value| value.as_object()) {
        if let Some(obj) = instance.as_object() {
            for (key, child_schema) in properties {
                if let Some(child_value) = obj.get(key) {
                    let child_path = format!("{instance_path}.{key}");
                    violations.extend(schema_violations(child_schema, child_value, &child_path));
                }
            }
            if schema_obj
                .get("additionalProperties")
                .and_then(|value| value.as_bool())
                == Some(false)
            {
                for key in obj.keys() {
                    if !properties.contains_key(key) {
                        violations.push(format!(
                            "{instance_path}: additional property `{key}` is not allowed"
                        ));
                    }
                }
            }
        }
    }

    if let Some(items) = schema_obj.get("items") {
        if let Some(values) = instance.as_array() {
            for (index, value) in values.iter().enumerate() {
                let child_path = format!("{instance_path}[{index}]");
                violations.extend(schema_violations(items, value, &child_path));
            }
        }
    }

    if let Some(all_of) = schema_obj.get("allOf").and_then(|value| value.as_array()) {
        for child_schema in all_of {
            violations.extend(schema_violations(child_schema, instance, instance_path));
        }
    }

    if let Some(not_schema) = schema_obj.get("not") {
        if matches_schema(not_schema, instance) {
            violations.push(format!("{instance_path}: value matches forbidden schema"));
        }
    }

    if let Some(if_schema) = schema_obj.get("if") {
        if matches_schema(if_schema, instance) {
            if let Some(then_schema) = schema_obj.get("then") {
                violations.extend(schema_violations(then_schema, instance, instance_path));
            }
        }
    }

    violations
}
