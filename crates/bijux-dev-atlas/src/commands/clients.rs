// SPDX-License-Identifier: Apache-2.0

use crate::cli::{ClientsCommand, ClientsCommandArgs, ClientsCompatMatrixCommand};
use crate::{emit_payload, resolve_repo_root};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const CLIENT_DOCS_CONFIG: &str = "configs/clients/atlas-client-docs.json";
const OPENAPI_SNAPSHOT: &str = "configs/openapi/v1/openapi.snapshot.json";

pub(crate) fn run_clients_command(quiet: bool, command: ClientsCommand) -> i32 {
    let run = match command {
        ClientsCommand::List(args) => run_clients_list(&args),
        ClientsCommand::Verify(args) => run_clients_verify(&args),
        ClientsCommand::DocsGenerate(args) => run_clients_docs_generate(&args),
        ClientsCommand::DocsVerify(args) => run_clients_docs_verify(&args),
        ClientsCommand::ExamplesVerify(args) => run_clients_examples_verify(&args),
        ClientsCommand::SchemaVerify(args) => run_clients_schema_verify(&args),
        ClientsCommand::CompatMatrix { command } => match command {
            ClientsCompatMatrixCommand::Verify(args) => run_clients_compat_matrix_verify(&args),
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
    let (_, docs_code) = run_clients_docs_verify(args)?;
    let (_, examples_code) = run_clients_examples_verify(args)?;
    let (_, schema_code) = run_clients_schema_verify(args)?;
    let (_, matrix_code) = run_clients_compat_matrix_verify(args)?;
    let passed = [docs_code, examples_code, schema_code, matrix_code]
        .iter()
        .filter(|code| **code == 0)
        .count();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "clients",
        "action": "verify",
        "client": args.client,
        "checks": [
            {"id": "docs-verify", "status": if docs_code == 0 {"pass"} else {"fail"}},
            {"id": "examples-verify", "status": if examples_code == 0 {"pass"} else {"fail"}},
            {"id": "schema-verify", "status": if schema_code == 0 {"pass"} else {"fail"}},
            {"id": "compat-matrix-verify", "status": if matrix_code == 0 {"pass"} else {"fail"}}
        ],
        "summary": {"total": 4, "passed": passed, "failed": 4 - passed}
    });
    Ok((emit_payload(args.format, args.out.clone(), &payload)?, if passed == 4 { 0 } else { 1 }))
}

fn run_clients_docs_generate(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let openapi_paths = load_openapi_paths(&repo_root)?;
    let docs_dir = repo_root.join("clients").join(&args.client).join("docs");
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
            format!("clients/{}/docs/index.md", args.client),
            format!("clients/{}/docs/api-reference.md", args.client),
            format!("clients/{}/docs/version-compatibility-matrix.md", args.client)
        ],
        "openapi_paths": openapi_paths.len(),
    });
    Ok((emit_payload(args.format, args.out.clone(), &payload)?, 0))
}

fn run_clients_docs_verify(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let paths = load_openapi_paths(&repo_root)?;
    let docs_dir = repo_root.join("clients").join(&args.client).join("docs");

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
    let examples_dir = repo_root.join("clients").join(&args.client).join("examples");
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

fn run_clients_schema_verify(args: &ClientsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let model = load_docs_model(&repo_root, &args.client)?;
    let docs_dir = repo_root.join("clients").join(&args.client).join("docs");
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
    let matrix_path = repo_root
        .join("clients")
        .join(&args.client)
        .join("docs/version-compatibility-matrix.md");
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
