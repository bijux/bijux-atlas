// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ClientsCommand, ClientsCommandArgs, ClientsCompatMatrixCommand, ClientsPythonCommand,
    ClientsPythonTestArgs,
};
use crate::{emit_payload, resolve_repo_root};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

const CLIENT_DOCS_CONFIG: &str = "configs/clients/atlas-client-docs.json";
const OPENAPI_SNAPSHOT: &str = "configs/openapi/v1/openapi.snapshot.json";

fn client_root(repo_root: &Path, client: &str) -> std::path::PathBuf {
    match client {
        "atlas-client" => repo_root.join("crates/bijux-atlas-client-python"),
        _ => repo_root.join("crates").join(client),
    }
}

pub(crate) fn run_clients_command(quiet: bool, command: ClientsCommand) -> i32 {
    let run = match command {
        ClientsCommand::List(args) => run_clients_list(&args),
        ClientsCommand::Verify(args) => run_clients_verify(&args),
        ClientsCommand::DocsGenerate(args) => run_clients_docs_generate(&args),
        ClientsCommand::DocsVerify(args) => run_clients_docs_verify(&args),
        ClientsCommand::ExamplesVerify(args) => run_clients_examples_verify(&args),
        ClientsCommand::ExamplesRun(args) => run_clients_examples_run(&args),
        ClientsCommand::SchemaVerify(args) => run_clients_schema_verify(&args),
        ClientsCommand::CompatMatrix { command } => match command {
            ClientsCompatMatrixCommand::Verify(args) => run_clients_compat_matrix_verify(&args),
        },
        ClientsCommand::Python { command } => match command {
            ClientsPythonCommand::Test(args) => run_clients_python_test(&args),
        },
    };

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas clients failed: {err}");
            1
        }
    }
}

fn run_clients_list(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let paths = load_openapi_paths(&repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "list",
        "client": args.client,
        "docs_entries": model.docs_entries.len(),
        "openapi_paths": paths.len(),
        "docs_config": CLIENT_DOCS_CONFIG
    });
    Ok((emit_payload(args.format, args.out.clone(), &payload)?, 0))
}

fn run_clients_verify(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let (docs_text, docs_code) = run_clients_docs_verify(args)?;
    let (examples_text, examples_code) = run_clients_examples_verify(args)?;
    let (schema_text, schema_code) = run_clients_schema_verify(args)?;
    let (matrix_text, matrix_code) = run_clients_compat_matrix_verify(args)?;
    let passed = [docs_code, examples_code, schema_code, matrix_code]
        .iter()
        .filter(|code| **code == 0)
        .count();
    if matches!(args.format, crate::cli::FormatArg::Text) && !args.markdown {
        let nextest = render_clients_verify_nextest(&[
            ("docs-verify", docs_code),
            ("examples-verify", examples_code),
            ("schema-verify", schema_code),
            ("compat-matrix-verify", matrix_code),
        ]);
        return Ok((nextest, if passed == 4 { 0 } else { 1 }));
    }
    let success = passed == 4;
    let evidence = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "verify-evidence",
        "client": args.client,
        "checks": [
            {"id": "docs-verify", "status": if docs_code == 0 {"pass"} else {"fail"}},
            {"id": "examples-verify", "status": if examples_code == 0 {"pass"} else {"fail"}},
            {"id": "schema-verify", "status": if schema_code == 0 {"pass"} else {"fail"}},
            {"id": "compat-matrix-verify", "status": if matrix_code == 0 {"pass"} else {"fail"}}
        ],
        "details": {
            "docs_verify": docs_text,
            "examples_verify": examples_text,
            "schema_verify": schema_text,
            "compat_matrix_verify": matrix_text
        },
        "summary": {"total": 4, "passed": passed, "failed": 4 - passed}
    });
    let evidence_root = repo_artifact_root(args, &args.client);
    fs::create_dir_all(&evidence_root)
        .map_err(|err| format!("failed to create {}: {err}", evidence_root.display()))?;
    let evidence_json_path = evidence_root.join("verify-evidence.json");
    fs::write(
        &evidence_json_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&evidence).map_err(|err| format!("serialize failed: {err}"))?
        ),
    )
    .map_err(|err| format!("failed to write {}: {err}", evidence_json_path.display()))?;
    let evidence_md_path = evidence_root.join("verify-evidence.md");
    fs::write(
        &evidence_md_path,
        render_clients_verify_markdown(&args.client, &evidence),
    )
    .map_err(|err| format!("failed to write {}: {err}", evidence_md_path.display()))?;

    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "verify",
        "client": args.client,
        "success": success,
        "summary": evidence["summary"].clone(),
        "checks": evidence["checks"].clone(),
        "evidence": {
            "json": evidence_json_path.display().to_string(),
            "markdown": evidence_md_path.display().to_string()
        }
    });
    Ok((emit_payload(args.format, args.out.clone(), &payload)?, if success { 0 } else { 1 }))
}

fn run_clients_docs_generate(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let openapi_paths = load_openapi_paths(&repo_root)?;
    let docs_dir = client_root(&repo_root, &args.client).join("docs");
    fs::create_dir_all(&docs_dir)
        .map_err(|err| format!("failed to create {}: {err}", docs_dir.display()))?;

    let index_text = render_index_markdown(&model, &openapi_paths);
    let api_reference_text = render_api_reference_markdown(&openapi_paths);
    let matrix_text = render_matrix_markdown(&model);
    fs::write(docs_dir.join("index.md"), index_text)
        .map_err(|err| format!("failed to write index.md: {err}"))?;
    fs::write(docs_dir.join("api-reference.md"), api_reference_text)
        .map_err(|err| format!("failed to write api-reference.md: {err}"))?;
    fs::write(docs_dir.join("version-compatibility-matrix.md"), matrix_text)
        .map_err(|err| format!("failed to write version-compatibility-matrix.md: {err}"))?;

    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "docs-generate",
        "client": args.client,
        "generated": [
            format!("crates/bijux-atlas-client-python/docs/index.md"),
            format!("crates/bijux-atlas-client-python/docs/api-reference.md"),
            format!("crates/bijux-atlas-client-python/docs/version-compatibility-matrix.md")
        ],
        "openapi_paths": openapi_paths.len(),
    });
    Ok((emit_payload(args.format, args.out.clone(), &payload)?, 0))
}

fn run_clients_docs_verify(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let paths = load_openapi_paths(&repo_root)?;
    let docs_dir = client_root(&repo_root, &args.client).join("docs");

    let expected = vec![
        (docs_dir.join("index.md"), render_index_markdown(&model, &paths)),
        (
            docs_dir.join("api-reference.md"),
            render_api_reference_markdown(&paths),
        ),
        (
            docs_dir.join("version-compatibility-matrix.md"),
            render_matrix_markdown(&model),
        ),
    ];
    let mut mismatches = Vec::new();
    for (path, expected_text) in expected {
        let actual = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        if normalize_newlines(&actual) != normalize_newlines(&expected_text) {
            mismatches.push(path.display().to_string());
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "docs-verify",
        "client": args.client,
        "success": mismatches.is_empty(),
        "mismatches": mismatches
    });
    Ok((
        emit_payload(args.format, args.out.clone(), &payload)?,
        if payload["success"].as_bool().unwrap_or(false) { 0 } else { 1 },
    ))
}

fn run_clients_examples_verify(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let examples_dir = client_root(&repo_root, &args.client).join("examples");
    let mut examples = Vec::new();
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(&examples_dir) {
        let entry = entry.map_err(|err| format!("failed to walk examples: {err}"))?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&repo_root)
            .unwrap_or(entry.path())
            .display()
            .to_string();
        let text = fs::read_to_string(entry.path())
            .map_err(|err| format!("failed to read {}: {err}", entry.path().display()))?;
        if text.contains("tools/generate_docs.py") {
            violations.push(format!("{rel}: references forbidden local docs script"));
        }
        examples.push(rel);
    }
    examples.sort();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "examples-verify",
        "client": args.client,
        "examples": examples,
        "violations": violations,
        "success": violations.is_empty()
    });
    Ok((
        emit_payload(args.format, args.out.clone(), &payload)?,
        if payload["success"].as_bool().unwrap_or(false) { 0 } else { 1 },
    ))
}

fn run_clients_examples_run(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let examples_dir = client_root(&repo_root, &args.client).join("examples");
    let mut ran = Vec::new();
    let mut failures = Vec::new();
    for entry in walkdir::WalkDir::new(&examples_dir) {
        let entry = entry.map_err(|err| format!("failed to walk examples: {err}"))?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&repo_root)
            .unwrap_or(entry.path())
            .display()
            .to_string();
        let text = fs::read_to_string(entry.path())
            .map_err(|err| format!("failed to read {}: {err}", entry.path().display()))?;
        let uses_runtime_surface = text.contains("/v1/query")
            || text.contains("dataset=\"genes\"")
            || text.contains("QueryRequest(");
        if !uses_runtime_surface {
            failures.push(format!("{rel}: missing runtime query surface usage"));
        }
        ran.push(rel);
    }
    ran.sort();
    let evidence = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "examples-run",
        "client": args.client,
        "examples_checked": ran,
        "violations": failures,
        "success": failures.is_empty()
    });
    let evidence_path = repo_artifact_root(args, &args.client).join("examples-run-evidence.json");
    if let Some(parent) = evidence_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(
        &evidence_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&evidence).map_err(|err| format!("serialize failed: {err}"))?
        ),
    )
    .map_err(|err| format!("failed to write {}: {err}", evidence_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "examples-run",
        "client": args.client,
        "success": failures.is_empty(),
        "evidence": evidence_path.display().to_string(),
        "violations": failures
    });
    Ok((
        emit_payload(args.format, args.out.clone(), &payload)?,
        if payload["success"].as_bool().unwrap_or(false) { 0 } else { 1 },
    ))
}

fn run_clients_python_test(args: &ClientsPythonTestArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let root = client_root(&repo_root, &args.common.client);
    let lock_path = root.join("requirements.lock");
    if !lock_path.exists() {
        return Err(format!(
            "missing deterministic lockfile {}; add requirements.lock",
            lock_path.display()
        ));
    }
    if args.install_deps {
        run_allowlisted_python(
            &root,
            &["-m", "pip", "install", "-r", "requirements.lock"],
            "install dependencies",
        )?;
    }

    let mut test_files = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("tests")) {
        let entry = entry.map_err(|err| format!("walk tests failed: {err}"))?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|v| v.to_str()) == Some("py")
            && entry
                .path()
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or_default()
                .starts_with("test_")
        {
            let rel = entry
                .path()
                .strip_prefix(&root)
                .unwrap_or(entry.path())
                .display()
                .to_string();
            test_files.push(rel);
        }
    }
    test_files.sort();
    let mut rows = Vec::new();
    for test_file in &test_files {
        let mut cmd_args = vec!["-m", "pytest", test_file.as_str()];
        if args.skip_network {
            cmd_args.extend(["-m", "not integration and not network"]);
        }
        let started = std::time::Instant::now();
        let status = run_allowlisted_python_status(&root, &cmd_args, args.skip_network)?;
        let duration_ms = started.elapsed().as_millis() as u64;
        rows.push((test_file.clone(), status.success(), duration_ms));
    }
    let passed = rows.iter().filter(|(_, ok, _)| *ok).count();
    let failed = rows.len().saturating_sub(passed);
    let evidence_root = repo_artifact_root(&args.common, &args.common.client);
    fs::create_dir_all(&evidence_root)
        .map_err(|err| format!("failed to create {}: {err}", evidence_root.display()))?;
    let evidence_path = evidence_root.join("python-test-evidence.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "python-test",
        "client": args.common.client,
        "skip_network": args.skip_network,
        "install_deps": args.install_deps,
        "lockfile": "requirements.lock",
        "summary": {"total": rows.len(), "passed": passed, "failed": failed},
        "tests": rows.iter().map(|(id, ok, duration_ms)| serde_json::json!({
            "id": id,
            "status": if *ok { "pass" } else { "fail" },
            "duration_ms": duration_ms
        })).collect::<Vec<_>>()
    });
    fs::write(
        &evidence_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&payload).map_err(|err| format!("serialize failed: {err}"))?
        ),
    )
    .map_err(|err| format!("failed to write {}: {err}", evidence_path.display()))?;
    if matches!(args.common.format, crate::cli::FormatArg::Text) && !args.common.markdown {
        let mut lines = vec!["clients python test".to_string()];
        for (idx, (id, ok, duration_ms)) in rows.iter().enumerate() {
            let status = if *ok { "PASS" } else { "FAIL" };
            lines.push(format!(
                "{status} ({}/{}) clients python {} [{} ms]",
                idx + 1,
                rows.len(),
                id,
                duration_ms
            ));
        }
        lines.push(format!(
            "summary: total={} passed={} failed={}",
            rows.len(),
            passed,
            failed
        ));
        return Ok((lines.join("\n"), if failed == 0 { 0 } else { 1 }));
    }
    let out = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "python-test",
        "client": args.common.client,
        "success": failed == 0,
        "summary": payload["summary"].clone(),
        "evidence": evidence_path.display().to_string()
    });
    Ok((
        emit_payload(args.common.format, args.common.out.clone(), &out)?,
        if failed == 0 { 0 } else { 1 },
    ))
}

fn run_clients_schema_verify(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let docs_dir = client_root(&repo_root, &args.client).join("docs");
    let mut missing = Vec::new();
    for entry in &model.docs_entries {
        if !docs_dir.join(&entry.path).exists() {
            missing.push(entry.path.clone());
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "schema-verify",
        "client": args.client,
        "success": missing.is_empty(),
        "missing_docs_entries": missing
    });
    Ok((
        emit_payload(args.format, args.out.clone(), &payload)?,
        if payload["success"].as_bool().unwrap_or(false) { 0 } else { 1 },
    ))
}

fn run_clients_compat_matrix_verify(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let matrix_path = client_root(&repo_root, &args.client).join("docs/version-compatibility-matrix.md");
    let matrix = fs::read_to_string(&matrix_path)
        .map_err(|err| format!("failed to read {}: {err}", matrix_path.display()))?;
    let required = [&model.python_sdk, &model.atlas_runtime, &model.api_surface];
    let missing = required
        .iter()
        .filter(|needle| !matrix.contains(needle.as_str()))
        .map(|needle| needle.to_string())
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "compat-matrix-verify",
        "client": args.client,
        "success": missing.is_empty(),
        "missing_values": missing
    });
    Ok((
        emit_payload(args.format, args.out.clone(), &payload)?,
        if payload["success"].as_bool().unwrap_or(false) { 0 } else { 1 },
    ))
}

#[derive(serde::Deserialize)]
struct ClientsDocsModel {
    client: String,
    python_sdk: String,
    atlas_runtime: String,
    api_surface: String,
    docs_entries: Vec<DocsEntry>,
}

#[derive(serde::Deserialize)]
struct DocsEntry {
    title: String,
    path: String,
}

fn load_docs_model(repo_root: &Path, client: &str) -> Result<ClientsDocsModel, String> {
    let path = repo_root.join(CLIENT_DOCS_CONFIG);
    let text =
        fs::read_to_string(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let model: ClientsDocsModel = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    if model.client != client {
        return Err(format!(
            "docs model client mismatch: expected `{client}`, found `{}`",
            model.client
        ));
    }
    if model.docs_entries.is_empty() {
        return Err("docs model must declare at least one docs entry".to_string());
    }
    Ok(model)
}

fn load_openapi_paths(repo_root: &Path) -> Result<Vec<String>, String> {
    let path = repo_root.join(OPENAPI_SNAPSHOT);
    let text =
        fs::read_to_string(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    let mut paths = value
        .get("paths")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "openapi snapshot must include `paths` object".to_string())?
        .keys()
        .filter(|p| p.starts_with("/v1/"))
        .cloned()
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn render_index_markdown(model: &ClientsDocsModel, openapi_paths: &[String]) -> String {
    let mut lines = vec!["# Python Client Documentation".to_string(), String::new()];
    for entry in &model.docs_entries {
        lines.push(format!("- [{}]({})", entry.title, entry.path));
    }
    lines.push(String::new());
    lines.push("## Supported Endpoints".to_string());
    lines.push(String::new());
    lines.push(format!(
        "Generated from `{OPENAPI_SNAPSHOT}` with {} public endpoints.",
        openapi_paths.len()
    ));
    lines.push(String::new());
    for path in openapi_paths.iter().take(12) {
        lines.push(format!("- `{path}`"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn render_api_reference_markdown(openapi_paths: &[String]) -> String {
    let mut lines = vec![
        "# Python Client API Reference".to_string(),
        String::new(),
        "Generated from repository OpenAPI snapshot.".to_string(),
        String::new(),
        "## Supported Runtime Endpoints".to_string(),
        String::new(),
    ];
    for path in openapi_paths {
        lines.push(format!("- `{path}`"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn render_matrix_markdown(model: &ClientsDocsModel) -> String {
    [
        "# Python Client Version Compatibility Matrix".to_string(),
        String::new(),
        "| atlas-client | Atlas runtime | API surface |".to_string(),
        "|---|---|---|".to_string(),
        format!(
            "| {} | {} | {} |",
            model.python_sdk, model.atlas_runtime, model.api_surface
        ),
        String::new(),
    ]
    .join("\n")
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_string()
}

fn run_allowlisted_python(cwd: &Path, args: &[&str], reason: &str) -> Result<(), String> {
    let status = run_allowlisted_python_status(cwd, args, false)?;
    if !status.success() {
        return Err(format!("python command failed while attempting to {reason}"));
    }
    Ok(())
}

fn run_allowlisted_python_status(
    cwd: &Path,
    args: &[&str],
    skip_network: bool,
) -> Result<std::process::ExitStatus, String> {
    let python = resolve_python_interpreter(cwd)?;
    if !cwd.ends_with(Path::new("crates/bijux-atlas-client-python")) {
        return Err(format!(
            "python execution outside allowed client crate is forbidden: {}",
            cwd.display()
        ));
    }
    let mut cmd = ProcessCommand::new(&python);
    cmd.current_dir(cwd).args(args);
    if skip_network {
        cmd.env("ATLAS_CLIENT_SKIP_NETWORK", "1");
    }
    cmd.status()
        .map_err(|err| format!("execute `{python}` failed: {err}"))
}

fn resolve_python_interpreter(cwd: &Path) -> Result<String, String> {
    for candidate in ["python3", "python"] {
        let status = ProcessCommand::new(candidate)
            .current_dir(cwd)
            .args(["--version"])
            .status();
        if matches!(status, Ok(s) if s.success()) {
            return Ok(candidate.to_string());
        }
    }
    Err("python interpreter not found (tried `python3` and `python`)".to_string())
}

fn repo_artifact_root(args: &ClientsCommandArgs, client: &str) -> PathBuf {
    let repo_root = resolve_repo_root(args.repo_root.clone()).unwrap_or_else(|_| PathBuf::from("."));
    repo_root.join("artifacts/clients").join(client)
}

fn render_clients_verify_markdown(client: &str, evidence: &serde_json::Value) -> String {
    let mut lines = vec![
        format!("# Client verification evidence: {client}"),
        String::new(),
        "| Check | Status |".to_string(),
        "|---|---|".to_string(),
    ];
    if let Some(checks) = evidence["checks"].as_array() {
        for check in checks {
            let id = check["id"].as_str().unwrap_or("unknown");
            let status = check["status"].as_str().unwrap_or("unknown");
            lines.push(format!("| `{id}` | `{status}` |"));
        }
    }
    lines.push(String::new());
    lines.push(format!(
        "Summary: total={} passed={} failed={}",
        evidence["summary"]["total"].as_u64().unwrap_or(0),
        evidence["summary"]["passed"].as_u64().unwrap_or(0),
        evidence["summary"]["failed"].as_u64().unwrap_or(0),
    ));
    lines.push(String::new());
    lines.join("\n")
}

fn render_clients_verify_nextest(rows: &[(&str, i32)]) -> String {
    let mut out = vec!["clients verify".to_string()];
    for (index, (id, code)) in rows.iter().enumerate() {
        let status = if *code == 0 { "PASS" } else { "FAIL" };
        out.push(format!("{status} ({}/{}) clients {}", index + 1, rows.len(), id));
    }
    let passed = rows.iter().filter(|(_, code)| *code == 0).count();
    out.push(format!(
        "summary: total={} passed={} failed={}",
        rows.len(),
        passed,
        rows.len() - passed
    ));
    out.join("\n")
}
