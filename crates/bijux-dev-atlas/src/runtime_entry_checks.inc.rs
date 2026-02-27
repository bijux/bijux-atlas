fn normalize_suite_name(raw: &str) -> Result<&str, String> {
    match raw {
        "ci-fast" => Ok("ci_fast"),
        "ci" => Ok("ci"),
        "local" => Ok("local"),
        "deep" => Ok("deep"),
        other => Ok(other),
    }
}

fn write_output_if_requested(out: Option<PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = out {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    }
    Ok(())
}

fn render_list_output(checks: &[CheckSpec], format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => {
            let mut lines = Vec::new();
            let mut current_domain = String::new();
            for check in checks {
                let domain = format!("{:?}", check.domain).to_ascii_lowercase();
                if domain != current_domain {
                    if !current_domain.is_empty() {
                        lines.push(String::new());
                    }
                    lines.push(format!("[{domain}]"));
                    current_domain = domain;
                }
                let tags = check
                    .tags
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                let suites = check
                    .suites
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "{}\tbudget_ms={}\ttags={}\tsuites={}\t{}",
                    check.id, check.budget_ms, tags, suites, check.title
                ));
            }
            Ok(lines.join("\n"))
        }
        FormatArg::Json => {
            let rows: Vec<serde_json::Value> = checks
                .iter()
                .map(|check| {
                    serde_json::json!({
                        "id": check.id.as_str(),
                        "domain": format!("{:?}", check.domain).to_ascii_lowercase(),
                        "tags": check.tags.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
                        "suites": check.suites.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
                        "budget_ms": check.budget_ms,
                        "title": check.title,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({"checks": rows}))
                .map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for list".to_string()),
    }
}

fn render_explain_output(explain_text: String, format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => Ok(explain_text),
        FormatArg::Json => {
            let mut map = serde_json::Map::new();
            for line in explain_text.lines() {
                if let Some((key, value)) = line.split_once(": ") {
                    map.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
            serde_json::to_string_pretty(&serde_json::Value::Object(map))
                .map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for explain".to_string()),
    }
}

pub(crate) struct CheckListOptions {
    repo_root: Option<PathBuf>,
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
    format: FormatArg,
    out: Option<PathBuf>,
}

pub(crate) fn run_check_list(options: CheckListOptions) -> Result<(String, i32), String> {
    let root = resolve_repo_root(options.repo_root)?;
    let selectors = parse_selectors(
        options.suite,
        options.domain,
        options.tag,
        options.id,
        options.include_internal,
        options.include_slow,
    )?;
    let registry = load_registry(&root)?;
    let checks = select_checks(&registry, &selectors)?;
    let rendered = render_list_output(&checks, options.format)?;
    write_output_if_requested(options.out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) fn run_check_explain(
    check_id: String,
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry = load_registry(&root)?;
    let id = CheckId::parse(&check_id)?;
    let rendered = render_explain_output(explain_output(&registry, &id)?, format)?;
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) struct CheckRunOptions {
    repo_root: Option<PathBuf>,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
    allow_subprocess: bool,
    allow_git: bool,
    allow_write: bool,
    allow_network: bool,
    fail_fast: bool,
    max_failures: Option<usize>,
    format: FormatArg,
    out: Option<PathBuf>,
    durations: usize,
}

#[derive(Debug, Serialize)]
struct DocsPageRow {
    path: String,
    in_nav: bool,
}

#[derive(Debug)]
struct DocsContext {
    repo_root: PathBuf,
    docs_root: PathBuf,
    artifacts_root: PathBuf,
    run_id: RunId,
}

#[derive(Default)]
struct DocsIssues {
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct ConfigsContext {
    repo_root: PathBuf,
    configs_root: PathBuf,
    artifacts_root: PathBuf,
    run_id: RunId,
}

pub(crate) fn run_check_run(options: CheckRunOptions) -> Result<(String, i32), String> {
    let root = resolve_repo_root(options.repo_root)?;
    let selectors = parse_selectors(
        options.suite,
        options.domain,
        options.tag,
        options.id,
        options.include_internal,
        options.include_slow,
    )?;
    let request = RunRequest {
        repo_root: root.clone(),
        domain: selectors.domain,
        capabilities: Capabilities::from_cli_flags(
            options.allow_write,
            options.allow_subprocess,
            options.allow_git,
            options.allow_network,
        ),
        artifacts_root: options
            .artifacts_root
            .or_else(|| Some(root.join("artifacts"))),
        run_id: options.run_id.map(|rid| RunId::parse(&rid)).transpose()?,
        command: Some("bijux dev atlas check run".to_string()),
    };
    let run_options = RunOptions {
        fail_fast: options.fail_fast,
        max_failures: options.max_failures,
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &run_options,
    )?;
    let rendered = match options.format {
        FormatArg::Text => render_text_with_durations(&report, options.durations),
        FormatArg::Json => render_json(&report)?,
        FormatArg::Jsonl => render_jsonl(&report)?,
    };
    write_output_if_requested(options.out, &rendered)?;
    Ok((rendered, exit_code_for_report(&report)))
}

pub(crate) fn run_workflows_command(quiet: bool, command: WorkflowsCommand) -> i32 {
    match command {
        WorkflowsCommand::Validate {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_run(CheckRunOptions {
            repo_root,
            artifacts_root: None,
            run_id: None,
            suite: None,
            domain: Some(DomainArg::Workflows),
            tag: None,
            id: None,
            include_internal,
            include_slow,
            allow_subprocess: false,
            allow_git: false,
            allow_write: false,
            allow_network: false,
            fail_fast: false,
            max_failures: None,
            format,
            out,
            durations: 0,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(
                    io::stderr(),
                    "bijux-dev-atlas workflows validate failed: {err}"
                );
                1
            }
        },
        WorkflowsCommand::Doctor {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_doctor(repo_root, include_internal, include_slow, format, out) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas workflows doctor failed: {err}");
                1
            }
        },
        WorkflowsCommand::Surface {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_list(CheckListOptions {
            repo_root,
            suite: None,
            domain: Some(DomainArg::Workflows),
            tag: None,
            id: None,
            include_internal,
            include_slow,
            format,
            out,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas workflows surface failed: {err}");
                1
            }
        },
    }
}

pub(crate) fn run_gates_command(quiet: bool, command: GatesCommand) -> i32 {
    match command {
        GatesCommand::List {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_list(CheckListOptions {
            repo_root,
            suite: None,
            domain: None,
            tag: None,
            id: None,
            include_internal,
            include_slow,
            format,
            out,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas gates list failed: {err}");
                1
            }
        },
        GatesCommand::Run {
            repo_root,
            artifacts_root,
            run_id,
            suite,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        } => match run_check_run(CheckRunOptions {
            repo_root,
            artifacts_root,
            run_id,
            suite: Some(suite),
            domain: None,
            tag: None,
            id: None,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas gates run failed: {err}");
                1
            }
        },
    }
}

pub(crate) fn run_check_doctor(
    repo_root: Option<PathBuf>,
    include_internal: bool,
    include_slow: bool,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry_report = registry_doctor(&root);
    let inventory_errors = validate_ops_inventory(&root);
    let selectors = parse_selectors(
        Some("doctor".to_string()),
        None,
        None,
        None,
        include_internal,
        include_slow,
    )?;
    let request = RunRequest {
        repo_root: root.clone(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some(RunId::from_seed("doctor_run")),
        command: Some("bijux dev atlas doctor".to_string()),
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &RunOptions::default(),
    )?;
    let docs_common = DocsCommonArgs {
        repo_root: Some(root.clone()),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some("doctor_docs".to_string()),
        format,
        out: None,
        allow_subprocess: false,
        allow_write: false,
        allow_network: false,
        strict: false,
        include_drafts: false,
    };
    let docs_ctx = docs_context(&docs_common)?;
    let docs_validate = docs_validate_payload(&docs_ctx, &docs_common)?;
    let docs_links = docs_links_payload(&docs_ctx, &docs_common)?;
    let docs_lint = docs_lint_payload(&docs_ctx, &docs_common)?;
    let configs_common = ConfigsCommonArgs {
        repo_root: Some(root.clone()),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some("doctor_configs".to_string()),
        format,
        out: None,
        allow_write: false,
        allow_subprocess: false,
        allow_network: false,
        strict: false,
    };
    let configs_ctx = configs_context(&configs_common)?;
    let configs_validate = configs_validate_payload(&configs_ctx, &configs_common)?;
    let configs_lint = configs_lint_payload(&configs_ctx, &configs_common)?;
    let configs_diff = configs_diff_payload(&configs_ctx, &configs_common)?;
    let check_exit = exit_code_for_report(&report);
    let inventory_error_count = inventory_errors.len();
    let ops_doctor_status = if inventory_errors.is_empty() && check_exit == 0 {
        "ok"
    } else {
        "failed"
    };
    let docs_error_count = docs_validate
        .get("errors")
        .and_then(|v| v.as_array())
        .map_or(0, Vec::len)
        + docs_links
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len)
        + docs_lint
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len);
    let configs_error_count = configs_validate
        .get("errors")
        .and_then(|v| v.as_array())
        .map_or(0, Vec::len)
        + configs_lint
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len)
        + configs_diff
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len);
    // Top-level doctor remains a stable fast governance health gate. Docs/configs summaries are
    // reported for visibility but do not fail the command by default because they contain broad
    // repo lint signals that are not part of the curated doctor contract.
    let status =
        if registry_report.errors.is_empty() && inventory_errors.is_empty() && check_exit == 0 {
            "ok"
        } else {
            "failed"
        };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "registry_errors": registry_report.errors,
        "inventory_errors": inventory_errors,
        "ops_doctor": {
            "status": ops_doctor_status,
            "inventory_errors": inventory_error_count,
            "checks_exit": check_exit
        },
        "docs_doctor": {
            "validate_errors": docs_validate.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "links_errors": docs_links.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "lint_errors": docs_lint.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "status": if docs_error_count == 0 { "ok" } else { "failed" }
        },
        "configs_doctor": {
            "validate_errors": configs_validate.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "lint_errors": configs_lint.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "diff_errors": configs_diff.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "status": if configs_error_count == 0 { "ok" } else { "failed" }
        },
        "control_plane_doctor": {
            "status": status,
            "ops": {"status": ops_doctor_status, "errors": inventory_error_count + usize::from(check_exit != 0)},
            "docs": {"status": if docs_error_count == 0 { "ok" } else { "failed" }, "errors": docs_error_count},
            "configs": {"status": if configs_error_count == 0 { "ok" } else { "failed" }, "errors": configs_error_count}
        },
        "check_report": report,
    });

    let evidence_dir = root.join("artifacts/atlas-dev/doctor");
    fs::create_dir_all(&evidence_dir)
        .map_err(|err| format!("failed to create {}: {err}", evidence_dir.display()))?;
    let evidence_path = evidence_dir.join("doctor.report.json");
    fs::write(
        &evidence_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", evidence_path.display()))?;

    let rendered = match format {
        FormatArg::Text => format!(
            "status: {status}\nregistry_errors: {}\ninventory_errors: {}\ncheck_summary: passed={} failed={} skipped={} errors={} total={}\nevidence: {}",
            payload["registry_errors"].as_array().map_or(0, Vec::len),
            payload["inventory_errors"].as_array().map_or(0, Vec::len),
            report.summary.passed,
            report.summary.failed,
            report.summary.skipped,
            report.summary.errors,
            report.summary.total,
            evidence_path.display(),
        ),
        FormatArg::Json => serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&payload).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    let exit = if status == "ok" { 0 } else { 1 };
    Ok((rendered, exit))
}

pub(crate) fn run_check_tree_budgets(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(repo_root)?;
    let mut errors = Vec::<String>::new();
    let mut deepest = Vec::<(String, usize)>::new();
    let forbidden_dir_names = ["misc", "tmp", "old", "legacy"];

    let rules = [
        ("configs", 4usize, 10usize),
        ("docs", 4usize, 10usize),
        ("ops", 5usize, usize::MAX),
    ];

    let exceptions_path = repo_root.join("configs/repo/tree-budget-exceptions.json");
    let exception_prefixes = if exceptions_path.exists() {
        let text = fs::read_to_string(&exceptions_path)
            .map_err(|e| format!("failed to read {}: {e}", exceptions_path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("failed to parse {}: {e}", exceptions_path.display()))?;
        value["allow_path_prefixes"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    for (root_name, max_depth, max_top_dirs) in rules {
        let root = repo_root.join(root_name);
        if !root.exists() {
            continue;
        }
        let top_level_dirs = fs::read_dir(&root)
            .map_err(|e| format!("failed to list {}: {e}", root.display()))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .count();
        if top_level_dirs > max_top_dirs {
            errors.push(format!(
                "TREE_BUDGET_ERROR: `{root_name}` top-level dirs {top_level_dirs} exceed budget {max_top_dirs}"
            ));
        }

        for file in walk_files_local(&root) {
            let rel = file
                .strip_prefix(&repo_root)
                .unwrap_or(&file)
                .display()
                .to_string();
            if exception_prefixes.iter().any(|prefix| rel.starts_with(prefix)) {
                continue;
            }
            let depth = file
                .strip_prefix(&root)
                .unwrap_or(&file)
                .components()
                .count();
            deepest.push((rel.clone(), depth));
            if depth > max_depth {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` depth {depth} exceeds `{root_name}` budget {max_depth}"
                ));
            }
            if (root_name == "configs" || root_name == "docs")
                && file
                    .components()
                    .any(|c| {
                        c.as_os_str()
                            .to_str()
                            .map(|name| forbidden_dir_names.contains(&name))
                            .unwrap_or(false)
                    })
            {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` uses forbidden directory name in configs/docs"
                ));
            }
        }
    }

    for root_name in ["configs", "docs"] {
        let root = repo_root.join(root_name);
        if !root.exists() {
            continue;
        }
        for dir in walk_files_local(&root)
            .into_iter()
            .filter_map(|p| p.parent().map(Path::to_path_buf))
            .collect::<std::collections::BTreeSet<_>>()
        {
            let rel_dir = dir
                .strip_prefix(&repo_root)
                .unwrap_or(&dir)
                .display()
                .to_string();
            if rel_dir.contains("/_generated") || rel_dir.contains("/_drafts") {
                continue;
            }
            let index_path = dir.join("INDEX.md");
            if !index_path.exists() {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: directory `{rel_dir}` is missing required `INDEX.md`"
                ));
            }
        }
    }

    let mut basename_paths = std::collections::BTreeMap::<String, Vec<String>>::new();
    for root_name in ["configs", "docs"] {
        for file in walk_files_local(&repo_root.join(root_name)) {
            let rel = file
                .strip_prefix(&repo_root)
                .unwrap_or(&file)
                .display()
                .to_string();
            if let Some(name) = file.file_name().and_then(|v| v.to_str()) {
                if matches!(name, "INDEX.md" | "README.md" | "OWNERS.md") {
                    continue;
                }
                basename_paths.entry(name.to_string()).or_default().push(rel);
            }
        }
    }
    for (name, paths) in basename_paths {
        if paths.len() > 1 {
            errors.push(format!(
                "TREE_BUDGET_ERROR: duplicate filename `{name}` across docs/configs: {}",
                paths.join(", ")
            ));
        }
    }

    let check_owner_coverage = |owners_path: &Path, prefix: &str| -> Result<Vec<String>, String> {
        let mut errs = Vec::<String>::new();
        if !owners_path.exists() {
            errs.push(format!(
                "TREE_BUDGET_ERROR: missing owners file `{}`",
                owners_path
                    .strip_prefix(&repo_root)
                    .unwrap_or(owners_path)
                    .display()
            ));
            return Ok(errs);
        }
        let text = fs::read_to_string(owners_path)
            .map_err(|e| format!("failed to read {}: {e}", owners_path.display()))?;
        let mut covered = std::collections::BTreeSet::<String>::new();
        for line in text.lines() {
            if let Some(idx) = line.find(&format!("`{prefix}/")) {
                let rest = &line[idx + 1..];
                if let Some(end) = rest.find('`') {
                    covered.insert(rest[..end].to_string());
                }
            }
        }
        let root = repo_root.join(prefix);
        if root.exists() {
            for entry in fs::read_dir(&root)
                .map_err(|e| format!("failed to list {}: {e}", root.display()))?
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
            {
                if let Some(name) = entry.file_name().to_str() {
                    let key = format!("{prefix}/{name}");
                    if !covered.contains(&key) {
                        errs.push(format!(
                            "TREE_BUDGET_ERROR: missing owner mapping for `{key}` in `{}`",
                            owners_path
                                .strip_prefix(&repo_root)
                                .unwrap_or(owners_path)
                                .display()
                        ));
                    }
                }
            }
        }
        Ok(errs)
    };
    errors.extend(check_owner_coverage(&repo_root.join("configs/OWNERS.md"), "configs")?);
    errors.extend(check_owner_coverage(&repo_root.join("docs/OWNERS.md"), "docs")?);

    let make_help = repo_root.join("make/help.md");
    let make_targets = repo_root.join("make/target-list.json");
    if make_help.exists() && make_targets.exists() {
        let help_text = fs::read_to_string(&make_help)
            .map_err(|e| format!("failed to read {}: {e}", make_help.display()))?;
        let targets_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&make_targets)
                .map_err(|e| format!("failed to read {}: {e}", make_targets.display()))?,
        )
        .map_err(|e| format!("failed to parse {}: {e}", make_targets.display()))?;
        for target in targets_json["public_targets"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
        {
            if !help_text.contains(&format!("- {target}:")) {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: public make target `{target}` missing from make/help.md"
                ));
            }
        }
    }

    for rel in [
        "docs/reference/commands.md",
        "docs/reference/schemas.md",
        "docs/reference/configs.md",
        "docs/reference/make-targets.md",
    ] {
        let path = repo_root.join(rel);
        if !path.exists() {
            errors.push(format!(
                "TREE_BUDGET_ERROR: missing required generated reference page `{rel}`"
            ));
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        if !text.contains("This page is generated by") {
            errors.push(format!(
                "TREE_BUDGET_ERROR: `{rel}` must declare generated artifact marker"
            ));
        }
    }

    let command_index_path = repo_root.join("docs/_generated/command-index.json");
    let known_command_ids = if command_index_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&command_index_path)
                .map_err(|e| format!("read {} failed: {e}", command_index_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", command_index_path.display()))?;
        value["commands"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| row["id"].as_str().map(str::to_string))
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        std::collections::BTreeSet::new()
    };

    let make_public_targets = if make_targets.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&make_targets)
                .map_err(|e| format!("read {} failed: {e}", make_targets.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", make_targets.display()))?;
        value["public_targets"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        std::collections::BTreeSet::new()
    };

    let schema_index_path = repo_root.join("docs/_generated/schema-index.json");
    let known_schema_paths = if schema_index_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&schema_index_path)
                .map_err(|e| format!("read {} failed: {e}", schema_index_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", schema_index_path.display()))?;
        value["schemas"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| row["path"].as_str().map(str::to_string))
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        std::collections::BTreeSet::new()
    };

    let command_ref_re =
        Regex::new(r"`bijux dev atlas ([a-z0-9_-]+(?: [a-z0-9_-]+)?)`").map_err(|e| e.to_string())?;
    let make_ref_re = Regex::new(r"`make ([a-z0-9_-]+)`").map_err(|e| e.to_string())?;
    let path_ref_re = Regex::new(r"`((?:configs|ops/schema)/[a-zA-Z0-9_./-]+)`")
        .map_err(|e| e.to_string())?;
    for doc in walk_files_local(&repo_root.join("docs"))
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
    {
        let rel = doc
            .strip_prefix(&repo_root)
            .unwrap_or(&doc)
            .display()
            .to_string();
        let text = fs::read_to_string(&doc).unwrap_or_default();
        for cap in command_ref_re.captures_iter(&text) {
            let ref_cmd = cap
                .get(1)
                .map(|m| m.as_str().replace(' ', "."))
                .unwrap_or_default();
            if !known_command_ids.is_empty() && !known_command_ids.contains(&ref_cmd) {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references unknown command `bijux dev atlas {}`",
                    cap.get(1).map(|m| m.as_str()).unwrap_or_default()
                ));
            }
        }
        for cap in make_ref_re.captures_iter(&text) {
            let ref_make = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            if !make_public_targets.is_empty() && !make_public_targets.contains(ref_make) {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references unknown make target `make {ref_make}`"
                ));
            }
        }
        for cap in path_ref_re.captures_iter(&text) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            if path.starts_with("configs/") && !repo_root.join(path).exists() {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references missing config path `{path}`"
                ));
            }
            if path.starts_with("ops/schema/")
                && !known_schema_paths.is_empty()
                && !known_schema_paths.contains(path)
            {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references schema path not present in schema index `{path}`"
                ));
            }
        }
    }

    deepest.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    deepest.truncate(20);

    let payload = serde_json::json!({
        "schema_version": 1,
        "text": if errors.is_empty() { "tree budgets passed" } else { "tree budgets failed" },
        "errors": errors,
        "deepest_paths": deepest
            .iter()
            .map(|(path, depth)| serde_json::json!({"path": path, "depth": depth}))
            .collect::<Vec<_>>(),
        "exceptions_file": if exceptions_path.exists() {
            serde_json::Value::String("configs/repo/tree-budget-exceptions.json".to_string())
        } else {
            serde_json::Value::Null
        }
    });

    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) { 1 } else { 0 }))
}

pub(crate) fn run_check_repo_doctor(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let (tree_rendered, tree_code) =
        run_check_tree_budgets(Some(root.clone()), FormatArg::Json, None)?;
    let tree_payload: serde_json::Value =
        serde_json::from_str(&tree_rendered).map_err(|e| format!("tree payload parse failed: {e}"))?;

    let docs_payload = docs_validate_payload(
        &docs_context(&DocsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            strict: true,
            include_drafts: false,
        })?,
        &DocsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            strict: true,
            include_drafts: false,
        },
    )?;
    let docs_code = if docs_payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
        1
    } else {
        0
    };

    let configs_payload = configs_validate_payload(
        &configs_context(&ConfigsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_write: false,
            allow_subprocess: false,
            allow_network: false,
            strict: true,
        })?,
        &ConfigsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_write: false,
            allow_subprocess: false,
            allow_network: false,
            strict: true,
        },
    )?;
    let configs_code = if configs_payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
        1
    } else {
        0
    };

    let mut docs_indexes = walk_files_local(&root.join("docs"))
        .into_iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(&root).ok()?.display().to_string();
            if rel.ends_with("/INDEX.md") || rel == "docs/INDEX.md" {
                Some(rel)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    docs_indexes.sort();
    let mut make_targets = Vec::<String>::new();
    let target_list_path = root.join("make/target-list.json");
    if target_list_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&target_list_path)
                .map_err(|e| format!("read {} failed: {e}", target_list_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", target_list_path.display()))?;
        make_targets = value["public_targets"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        make_targets.sort();
    }
    let mut config_groups = fs::read_dir(root.join("configs"))
        .map_err(|e| format!("list configs failed: {e}"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .collect::<Vec<_>>();
    config_groups.sort();

    let snapshot = serde_json::json!({
        "schema_version": 1,
        "make_public_targets": make_targets,
        "docs_indexes": docs_indexes,
        "config_groups": config_groups
    });
    let snapshot_rel = Path::new("configs/repo/surface-snapshot.json");
    let mut snapshot_drift_error = serde_json::Value::Null;
    let snapshot_path = root.join(snapshot_rel);
    if snapshot_path.exists() {
        let expected: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&snapshot_path)
                .map_err(|e| format!("read {} failed: {e}", snapshot_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", snapshot_path.display()))?;
        if expected != snapshot {
            snapshot_drift_error = serde_json::json!(format!(
                "REPO_SURFACE_DRIFT_ERROR: `{}` does not match current repo surface snapshot",
                snapshot_rel.display()
            ));
        }
    } else {
        snapshot_drift_error = serde_json::json!(format!(
            "REPO_SURFACE_DRIFT_ERROR: missing `{}`",
            snapshot_rel.display()
        ));
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "text": if tree_code == 0 && docs_code == 0 && configs_code == 0 && snapshot_drift_error.is_null() { "repo doctor passed" } else { "repo doctor failed" },
        "checks": {
            "tree_budgets": tree_payload,
            "docs_validate": docs_payload,
            "configs_validate": configs_payload
        },
        "surface_snapshot": snapshot,
        "surface_snapshot_contract": snapshot_rel.display().to_string(),
        "surface_snapshot_drift_error": snapshot_drift_error
    });
    let rendered = emit_payload(format, out, &payload)?;
    let code = if tree_code == 0
        && docs_code == 0
        && configs_code == 0
        && payload["surface_snapshot_drift_error"].is_null()
    {
        0
    } else {
        1
    };
    Ok((rendered, code))
}

pub(crate) fn run_check_registry_doctor(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let report = registry_doctor(&root);
    let status = if report.errors.is_empty() {
        "ok"
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "repo_root": root.display().to_string(),
        "errors": report.errors,
    });
    let rendered = match format {
        FormatArg::Text => format!(
            "status: {status}\nerrors: {}",
            payload["errors"].as_array().map_or(0, Vec::len)
        ),
        FormatArg::Json => serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&payload).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

