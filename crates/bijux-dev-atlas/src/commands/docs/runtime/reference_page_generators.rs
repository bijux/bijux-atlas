fn docs_reference_generate_or_check(
    repo_root: &std::path::Path,
    allow_write: bool,
) -> Result<(Vec<String>, Vec<String>), String> {
    let ref_dir = repo_root.join("docs/operations/reference");
    let mut changed = Vec::<String>::new();
    let mut generated = Vec::<String>::new();
    let targets = docs_reference_target_contents(repo_root)?;
    for (rel, content) in targets {
        let path = repo_root.join(rel);
        generated.push(rel.to_string());
        let existing = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {} failed: {e}", path.display()))?;
        if existing != content {
            changed.push(rel.to_string());
            if allow_write {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
                }
                std::fs::write(&path, content)
                    .map_err(|e| format!("write {} failed: {e}", path.display()))?;
            }
        }
    }
    let _ = ref_dir;
    Ok((changed, generated))
}

fn docs_reference_target_contents(
    repo_root: &std::path::Path,
) -> Result<Vec<(&'static str, String)>, String> {
    Ok(vec![
        (
            "docs/operations/reference/commands.md",
            render_docs_reference_commands(repo_root)?,
        ),
        (
            "docs/reference/commands.md",
            render_docs_reference_commands_registry(repo_root)?,
        ),
        (
            "docs/reference/schemas.md",
            render_docs_reference_schemas(repo_root)?,
        ),
        (
            "docs/reference/configs.md",
            render_docs_reference_configs(repo_root)?,
        ),
        (
            "docs/reference/make-targets.md",
            render_docs_reference_make_targets(repo_root)?,
        ),
        (
            "docs/reference/repo-map.md",
            render_docs_reference_repo_map(repo_root)?,
        ),
        (
            "docs/operations/reference/ops-surface.md",
            render_docs_reference_ops_surface(repo_root)?,
        ),
        (
            "docs/operations/reference/tools.md",
            render_docs_reference_tools(repo_root)?,
        ),
        (
            "docs/operations/reference/toolchain.md",
            render_docs_reference_toolchain(repo_root)?,
        ),
        (
            "docs/operations/reference/pins.md",
            render_docs_reference_pins(repo_root)?,
        ),
        (
            "docs/operations/reference/gates.md",
            render_docs_reference_gates(repo_root)?,
        ),
        (
            "docs/operations/reference/drills.md",
            render_docs_reference_drills(repo_root)?,
        ),
        (
            "docs/operations/reference/schema-index.md",
            render_docs_reference_schema_index(),
        ),
        (
            "docs/operations/reference/evidence-model.md",
            render_docs_reference_evidence_model(repo_root)?,
        ),
        (
            "docs/operations/reference/what-breaks-if-removed.md",
            render_docs_reference_what_breaks(repo_root)?,
        ),
    ])
}

fn render_docs_reference_commands_registry(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("docs/_generated/command-index.json"))
            .map_err(|e| format!("read command-index.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse command-index.json failed: {e}"))?;
    let mut rows = value["commands"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| {
            (
                row["id"].as_str().unwrap_or_default().to_string(),
                row["domain"].as_str().unwrap_or_default().to_string(),
                row["summary"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();

    let mut out = String::new();
    out.push_str("# Commands Reference\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n");
    out.push_str("- Tier: `generated`\n");
    out.push_str("- Audience: `operators`\n");
    out.push_str("- Source-of-truth: `docs/_generated/command-index.json`\n\n");
    out.push_str("## Commands\n\n| Command ID | Domain | Summary |\n| --- | --- | --- |\n");
    for (id, domain, summary) in rows {
        out.push_str(&format!("| `{id}` | `{domain}` | {summary} |\n"));
    }
    Ok(out)
}

fn render_docs_reference_schemas(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("docs/_generated/schema-index.json"))
            .map_err(|e| format!("read schema-index.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse schema-index.json failed: {e}"))?;
    let mut rows = value["schemas"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| {
            (
                row["path"].as_str().unwrap_or_default().to_string(),
                row["title"].as_str().unwrap_or_default().to_string(),
                row["kind"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();

    let mut out = String::new();
    out.push_str("# Schemas Reference\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n");
    out.push_str("- Tier: `generated`\n");
    out.push_str("- Audience: `operators`\n");
    out.push_str("- Source-of-truth: `docs/_generated/schema-index.json`\n\n");
    out.push_str("## Schemas\n\n| Path | Title | Kind |\n| --- | --- | --- |\n");
    for (path, title, kind) in rows {
        out.push_str(&format!("| `{path}` | {title} | `{kind}` |\n"));
    }
    Ok(out)
}

fn render_docs_reference_configs(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("configs/inventory/consumers.json"))
            .map_err(|e| format!("read configs inventory failed: {e}"))?,
    )
    .map_err(|e| format!("parse configs inventory failed: {e}"))?;
    let mut rows = value["groups"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|(group, consumers)| {
            let joined = consumers
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            (group, joined)
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|(group, _)| group.clone());

    let mut out = String::new();
    out.push_str("# Configs Reference\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n");
    out.push_str("- Tier: `generated`\n");
    out.push_str("- Audience: `operators`\n");
    out.push_str("- Source-of-truth: `configs/inventory/consumers.json`\n\n");
    out.push_str("## Config Groups\n\n| Group | Consumers |\n| --- | --- |\n");
    for (group, consumers) in rows {
        out.push_str(&format!("| `{group}` | `{consumers}` |\n"));
    }
    Ok(out)
}

fn render_docs_reference_make_targets(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("configs/ops/make-target-registry.json"))
            .map_err(|e| format!("read make target registry failed: {e}"))?,
    )
    .map_err(|e| format!("parse make target registry failed: {e}"))?;
    let mut rows = value["targets"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| {
            (
                row["name"].as_str().unwrap_or_default().to_string(),
                row["surface"].as_str().unwrap_or_default().to_string(),
                row["description"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();

    let mut out = String::new();
    out.push_str("# Make Targets Reference\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n");
    out.push_str("- Tier: `generated`\n");
    out.push_str("- Audience: `operators`\n");
    out.push_str("- Source-of-truth: `configs/ops/make-target-registry.json`\n\n");
    out.push_str("## Targets\n\n| Target | Surface | Description |\n| --- | --- | --- |\n");
    for (name, surface, description) in rows {
        out.push_str(&format!("| `{name}` | `{surface}` | {description} |\n"));
    }
    Ok(out)
}

fn render_docs_reference_repo_map(repo_root: &std::path::Path) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("# Repository Map\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n");
    out.push_str("- Tier: `generated`\n");
    out.push_str("- Audience: `operators`\n");
    out.push_str("- Source-of-truth: repository filesystem snapshot\n\n");
    out.push_str("## Top-Level Directories\n\n| Directory | Markdown Files | Total Files |\n| --- | --- | --- |\n");

    let mut dirs = std::fs::read_dir(repo_root)
        .map_err(|e| format!("list repo root failed: {e}"))?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect::<Vec<_>>();
    dirs.sort();
    for dir in dirs {
        if dir.starts_with(".git") {
            continue;
        }
        let root = repo_root.join(&dir);
        let files = walk_files_local(root.as_path());
        let md_count = files
            .iter()
            .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
            .count();
        out.push_str(&format!("| `{dir}` | `{md_count}` | `{}` |\n", files.len()));
    }
    Ok(out)
}

fn run_bijux_dev_atlas_help(repo_root: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .current_dir(repo_root)
        .args(["run", "-q", "-p", "bijux-dev-atlas", "--"])
        .args(args)
        .output()
        .map_err(|e| format!("spawn cargo for docs reference help failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo help command failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8(output.stdout)
        .map_err(|e| format!("invalid utf8 in help output: {e}"))?
        .trim_end()
        .to_string())
}

fn trim_help_usage_and_commands(help: &str) -> String {
    let mut out = Vec::new();
    for line in help.lines() {
        if line.starts_with("Options:") {
            break;
        }
        out.push(line.trim_end());
    }
    while out.last().is_some_and(|line| line.is_empty()) {
        out.pop();
    }
    out.join("\n")
}

fn render_docs_reference_commands(repo_root: &std::path::Path) -> Result<String, String> {
    let root_help = trim_help_usage_and_commands(&run_bijux_dev_atlas_help(repo_root, &["--help"])?);
    let ops_help = trim_help_usage_and_commands(&run_bijux_dev_atlas_help(repo_root, &["ops", "--help"])?);
    Ok(format!(
        "# Command Surface Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `bijux dev atlas --help`, `bijux dev atlas ops --help`, `make/makefiles/GENERATED_TARGETS.md`\n\n## Purpose\n\nGenerated reference for the supported command surface. Narrative docs should link here instead of restating command lists.\n\n## bijux-dev-atlas\n\n```text\n{root_help}\n```\n\n## bijux-dev-atlas ops\n\n```text\n{ops_help}\n```\n\n## Make Wrapper Surface\n\nSee `make/makefiles/GENERATED_TARGETS.md` and generated ops surface references. Narrative docs must not duplicate long `make ops-*` command lists.\n\n## Regenerate\n\n- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`\n"
    ))
}

fn render_docs_reference_ops_surface(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/surfaces.json"))
            .map_err(|e| format!("read surfaces.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse surfaces.json failed: {e}"))?;
    let mut entrypoints = value["entrypoints"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    entrypoints.sort();
    let mut commands = value["bijux-dev-atlas_commands"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    commands.sort();
    let mut actions = value["actions"].as_array().cloned().unwrap_or_default();
    actions.sort_by_key(|row| row["id"].as_str().unwrap_or_default().to_string());
    let mut out = String::new();
    out.push_str("# Ops Surface Reference\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/surfaces.json`, `ops/_generated.example/control-plane.snapshot.md`\n\n");
    out.push_str("## Purpose\n\nGenerated ops surface reference derived from inventory surfaces.\n\n");
    out.push_str("## Entry Points\n\n");
    for item in entrypoints {
        out.push_str(&format!("- `{item}`\n"));
    }
    out.push_str("\n## bijux-dev-atlas Commands\n\n");
    for item in commands {
        out.push_str(&format!("- `{item}`\n"));
    }
    out.push_str("\n## Actions\n\n");
    for item in actions {
        let encoded = serde_json::to_string(&item)
            .map_err(|e| format!("encode action row for ops surface reference failed: {e}"))?;
        out.push_str(&format!("- `{encoded}`\n"));
    }
    out.push_str("\n## See Also\n\n- `ops/_generated.example/control-plane.snapshot.md` (example generated snapshot)\n- `ops/inventory/surfaces.json` (machine truth)\n");
    Ok(out)
}

fn render_docs_reference_tools(repo_root: &std::path::Path) -> Result<String, String> {
    let text = std::fs::read_to_string(repo_root.join("ops/inventory/tools.toml"))
        .map_err(|e| format!("read tools.toml failed: {e}"))?;
    let value: toml::Value = toml::from_str(&text).map_err(|e| format!("parse tools.toml failed: {e}"))?;
    let mut rows = value["tools"].as_array().cloned().unwrap_or_default();
    rows.sort_by_key(|row| row["name"].as_str().unwrap_or_default().to_string());
    let mut out = String::new();
    out.push_str("# Tools Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/tools.toml`\n\n## Tools\n\n| Tool | Required | Probe Args | Version Regex |\n| --- | --- | --- | --- |\n");
    for row in rows {
        let probe = row["probe_argv"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` |\n",
            row["name"].as_str().unwrap_or_default(),
            row["required"].as_bool().unwrap_or(false),
            probe,
            row["version_regex"].as_str().unwrap_or_default()
        ));
    }
    out.push_str("\n## Regenerate\n\n- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`\n");
    Ok(out)
}

fn render_docs_reference_toolchain(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/toolchain.json"))
            .map_err(|e| format!("read toolchain.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse toolchain.json failed: {e}"))?;
    let mut out = String::new();
    out.push_str("# Toolchain Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/toolchain.json`\n\n## Tools\n\n| Tool | Required | Probe Args |\n| --- | --- | --- |\n");
    let mut tools = value["tools"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    tools.sort_by_key(|(k, _)| k.clone());
    for (name, row) in tools {
        let probe = row["probe_argv"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            name,
            row["required"].as_bool().unwrap_or(false),
            probe
        ));
    }
    out.push_str("\n## Images\n\n| Image Key | Reference |\n| --- | --- |\n");
    let mut images = value["images"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    images.sort_by_key(|(k, _)| k.clone());
    for (k, v) in images {
        out.push_str(&format!("| `{}` | `{}` |\n", k, v.as_str().unwrap_or_default()));
    }
    out.push_str("\n## GitHub Actions Pins\n\n| Action | Ref | SHA |\n| --- | --- | --- |\n");
    let mut actions = value["github_actions"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    actions.sort_by_key(|(k, _)| k.clone());
    for (action, row) in actions {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            action,
            row["ref"].as_str().unwrap_or_default(),
            row["sha"].as_str().unwrap_or_default()
        ));
    }
    Ok(out)
}

fn render_docs_reference_pins(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/pins.yaml"))
            .map_err(|e| format!("read pins.yaml failed: {e}"))?,
    )
    .map_err(|e| format!("parse pins.yaml failed: {e}"))?;
    let mut rows = Vec::<(String, String, String)>::new();
    collect_yaml_rows("root", &value, &mut rows);
    rows.sort();
    let mut out = String::new();
    out.push_str("# Pins Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/pins.yaml`\n\n## Pins\n\n| Section | Key | Value |\n| --- | --- | --- |\n");
    for (section, key, val) in rows {
        out.push_str(&format!("| `{}` | `{}` | `{}` |\n", section, key, val));
    }
    Ok(out)
}

fn collect_yaml_rows(prefix: &str, value: &serde_yaml::Value, out: &mut Vec<(String, String, String)>) {
    if let serde_yaml::Value::Mapping(map) = value {
        for (k, v) in map {
            let key = k.as_str().unwrap_or_default();
            if prefix == "root" && !matches!(v, serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_)) {
                out.push((prefix.to_string(), key.to_string(), yaml_scalar_string(v)));
            } else {
                match v {
                    serde_yaml::Value::Mapping(inner) => {
                        for (ik, iv) in inner {
                            out.push((
                                key.to_string(),
                                ik.as_str().unwrap_or_default().to_string(),
                                yaml_scalar_string(iv),
                            ));
                        }
                    }
                    serde_yaml::Value::Sequence(seq) => {
                        for (idx, item) in seq.iter().enumerate() {
                            out.push((key.to_string(), idx.to_string(), yaml_scalar_string(item)));
                        }
                    }
                    _ => out.push((prefix.to_string(), key.to_string(), yaml_scalar_string(v))),
                }
            }
        }
    }
}

fn yaml_scalar_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        _ => serde_yaml::to_string(v).unwrap_or_default().trim().to_string(),
    }
}

fn render_docs_reference_gates(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/gates.json"))
            .map_err(|e| format!("read gates.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse gates.json failed: {e}"))?;
    let mut gates = value["gates"].as_array().cloned().unwrap_or_default();
    gates.sort_by_key(|g| g["id"].as_str().unwrap_or_default().to_string());
    let mut out = String::new();
    out.push_str("# Gates Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/gates.json`\n\n## Gates\n\n| Gate ID | Category | Action ID | Description |\n| --- | --- | --- | --- |\n");
    for gate in gates {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            gate["id"].as_str().unwrap_or_default(),
            gate["category"].as_str().unwrap_or_default(),
            gate["action_id"].as_str().unwrap_or_default(),
            gate["description"].as_str().unwrap_or_default()
        ));
    }
    Ok(out)
}

fn render_docs_reference_drills(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/drills.json"))
            .map_err(|e| format!("read drills.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse drills.json failed: {e}"))?;
    let mut drills = value["drills"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().map(ToString::to_string).unwrap_or_else(|| v["id"].as_str().unwrap_or_default().to_string()))
        .collect::<Vec<_>>();
    drills.sort();
    let mut out = String::new();
    out.push_str("# Drills Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/drills.json`\n\n## Drills\n\n");
    for drill in drills {
        out.push_str(&format!("- `{drill}`\n"));
    }
    Ok(out)
}

fn render_docs_reference_schema_index() -> String {
    "# Schema Index Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/schema/generated/schema-index.md`\n\n## Canonical Source\n\n- `ops/schema/generated/schema-index.md` is the authoritative generated schema index.\n- This page is a docs-site reference pointer to avoid duplicating the schema table.\n".to_string()
}

fn render_docs_reference_evidence_model(repo_root: &std::path::Path) -> Result<String, String> {
    let levels: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/report/evidence-levels.schema.json"))
            .map_err(|e| format!("read evidence-levels schema failed: {e}"))?,
    )
    .map_err(|e| format!("parse evidence-levels schema failed: {e}"))?;
    let _bundle: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/report/release-evidence-bundle.schema.json"))
            .map_err(|e| format!("read release-evidence-bundle schema failed: {e}"))?,
    )
    .map_err(|e| format!("parse release-evidence-bundle schema failed: {e}"))?;
    Ok(format!(
        "# Evidence Model Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/schema/report/evidence-levels.schema.json`, `ops/schema/report/release-evidence-bundle.schema.json`\n\n## Canonical Schemas\n\n- `ops/schema/report/evidence-levels.schema.json`\n- `ops/schema/report/release-evidence-bundle.schema.json`\n\n## Notes\n\n- evidence-levels schema title: `{}`\n",
        levels["title"].as_str().unwrap_or_default()
    ))
}

fn render_docs_reference_what_breaks(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/_generated.example/what-breaks-if-removed-report.json"))
            .map_err(|e| format!("read what-breaks-if-removed report failed: {e}"))?,
    )
    .map_err(|e| format!("parse what-breaks-if-removed report failed: {e}"))?;
    let mut out = String::new();
    out.push_str("# What Breaks If Removed Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/_generated.example/what-breaks-if-removed-report.json`\n\n## Removal Impact Targets\n\n| Path | Impact | Consumers |\n| --- | --- | --- |\n");
    for row in value["targets"].as_array().into_iter().flatten() {
        let consumers = row["consumers"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            row["path"].as_str().unwrap_or_default(),
            row["impact"].as_str().unwrap_or_default(),
            consumers
        ));
    }
    Ok(out)
}
