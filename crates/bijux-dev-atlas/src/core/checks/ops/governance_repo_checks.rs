// SPDX-License-Identifier: Apache-2.0

use super::*;
use serde_json::Value;

fn read_json_value(path: &Path) -> Result<Value, CheckError> {
    let text = fs::read_to_string(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))
}

pub(super) fn check_root_no_scripts_areas_presence_or_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let root = Path::new("scripts/areas");
    if ctx.adapters.fs.exists(ctx.repo_root, root) {
        violations.push(violation(
            "ROOT_SCRIPTS_AREAS_DIRECTORY_PRESENT",
            "scripts/areas directory exists".to_string(),
            "remove scripts/areas; route control-plane through bijux dev atlas",
            Some(root),
        ));
    }
    for rel in ["makefiles", ".github/workflows", "docs", "ops"] {
        violations.extend(check_no_string_references_under(
            ctx,
            rel,
            "scripts/areas",
            "ROOT_SCRIPTS_AREAS_REFERENCE_FOUND",
        )?);
    }
    Ok(violations)
}

pub(super) fn check_root_forbidden_retired_directories_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel in ["scripts", "tools", "xtask"] {
        let path = Path::new(rel);
        if ctx.adapters.fs.exists(ctx.repo_root, path) {
            violations.push(violation(
                "ROOT_FORBIDDEN_LEGACY_DIRECTORY_PRESENT",
                format!("forbidden retired top-level directory exists: {}", path.display()),
                "delete the directory and move behavior into crates/bijux-dev-atlas command surfaces",
                Some(path),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_root_makefile_single_include_entrypoint(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("Makefile");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>();
    if lines.len() == 2
        && lines[0] == "include make/public.mk"
        && lines[1] == "include make/help.mk"
    {
        return Ok(Vec::new());
    }
    if !lines.contains(&"include make/public.mk") || !lines.contains(&"include make/help.mk") {
        return Ok(vec![violation(
            "ROOT_MAKEFILE_MISSING_ROOT_INCLUDE",
            "root Makefile must include make/public.mk and make/help.mk".to_string(),
            "use root Makefile as a thin include entrypoint",
            Some(rel),
        )]);
    }
    lines.retain(|line| *line != "include make/public.mk" && *line != "include make/help.mk");
    Ok(vec![violation(
        "ROOT_MAKEFILE_NOT_SINGLE_INCLUDE_ENTRYPOINT",
        "root Makefile contains logic beyond the include-only entrypoint".to_string(),
        "keep root Makefile to two includes only: `include make/public.mk` and `include make/help.mk`",
        Some(rel),
    )])
}

pub(super) fn check_makefiles_root_includes_sorted(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/root.mk");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let includes = text
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("include "))
        .map(|line| line.trim_start_matches("include ").to_string())
        .collect::<Vec<_>>();
    let mut sorted = includes.clone();
    sorted.sort();
    if includes == sorted {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "MAKEFILES_ROOT_INCLUDES_NOT_SORTED",
            "make/root.mk include statements must be sorted".to_string(),
            "sort include lines lexicographically for deterministic diffs",
            Some(rel),
        )])
    }
}

pub(super) fn check_root_top_level_directories_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let expected = [
        "artifacts",
        "configs",
        "crates",
        "docker",
        "docs",
        "make",
        "ops",
    ];
    let mut actual = fs::read_dir(ctx.repo_root)
        .map_err(|err| CheckError::Failed(err.to_string()))?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect::<Vec<_>>();
    actual.sort();
    let expected_vec = expected.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if actual == expected_vec {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_TOP_LEVEL_DIRECTORIES_CONTRACT_MISMATCH",
            format!("top-level visible directories mismatch: actual={actual:?} expected={expected_vec:?}"),
            "keep only the canonical top-level directory set and move retired roots into crates/ or ops/",
            None,
        )])
    }
}

pub(super) fn check_root_cargo_config_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".cargo/config.toml");
    let path = ctx.repo_root.join(rel);
    let text = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in ["[build]", "target-dir = \"artifacts/target\"", "[term]"] {
        if !text.contains(required) {
            violations.push(violation(
                "ROOT_CARGO_CONFIG_CONTRACT_MISSING",
                format!(".cargo/config.toml is missing required contract snippet: `{required}`"),
                "restore deterministic cargo target-dir and terminal config defaults",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_root_rust_toolchain_toml_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("rust-toolchain.toml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in [
        "[toolchain]",
        "channel = \"",
        "profile = \"minimal\"",
        "components = [\"rustfmt\", \"clippy\"]",
    ] {
        if !text.contains(required) {
            violations.push(violation(
                "ROOT_RUST_TOOLCHAIN_CONTRACT_MISSING",
                format!("rust-toolchain.toml is missing required contract snippet: `{required}`"),
                "pin stable Rust channel with minimal profile and rustfmt/clippy components",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_root_rustfmt_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/rust/rustfmt.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_RUSTFMT_TOML_MISSING",
            "configs/rust/rustfmt.toml must exist".to_string(),
            "define rustfmt policy under configs/rust and use explicit cargo fmt --config-path",
            Some(rel),
        )])
    }
}

pub(super) fn check_root_clippy_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/rust/clippy.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_CLIPPY_TOML_MISSING",
            "configs/rust/clippy.toml must exist".to_string(),
            "define clippy policy under configs/rust and use explicit CLIPPY_CONF_DIR",
            Some(rel),
        )])
    }
}

pub(super) fn check_configs_nextest_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/nextest/nextest.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CONFIGS_NEXTEST_TOML_MISSING",
            "configs/nextest/nextest.toml must exist".to_string(),
            "define nextest execution profiles and isolated store path under configs/nextest",
            Some(rel),
        )])
    }
}

pub(super) fn check_configs_security_deny_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/security/deny.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CONFIGS_SECURITY_DENY_TOML_MISSING",
            "configs/security/deny.toml must exist".to_string(),
            "keep cargo-deny policy under configs/security/deny.toml",
            Some(rel),
        )])
    }
}

pub(super) fn check_workflows_rust_toolchain_matches_repo_pin(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let toolchain_rel = Path::new("rust-toolchain.toml");
    let workflow_rel = Path::new(".github/workflows/ci-pr.yml");
    let toolchain_text = fs::read_to_string(ctx.repo_root.join(toolchain_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let workflow_text = fs::read_to_string(ctx.repo_root.join(workflow_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let pinned = toolchain_text
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("channel = ") {
                trimmed.split('"').nth(1).map(str::to_string)
            } else {
                None
            }
        })
        .ok_or_else(|| CheckError::Failed("rust-toolchain.toml channel missing".to_string()))?;
    let expected = format!("toolchain: {pinned}");
    if workflow_text.contains(&expected) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "WORKFLOWS_RUST_TOOLCHAIN_PIN_MISMATCH",
            format!(
                "ci-pr workflow must pin the same Rust toolchain as rust-toolchain.toml (`{pinned}`)"
            ),
            "update .github/workflows/ci-pr.yml dtolnay/rust-toolchain step `with.toolchain` to match rust-toolchain.toml",
            Some(workflow_rel),
        )])
    }
}

pub(super) fn check_crates_bijux_atlas_cli_owns_umbrella_dispatch(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let crates_root = ctx.repo_root.join("crates");
    if !crates_root.exists() {
        return Ok(Vec::new());
    }
    let mut owners = BTreeSet::new();
    for file in walk_files(&crates_root) {
        if file.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        if rel.starts_with("crates/bijux-dev-atlas/")
            || rel == Path::new("crates/bijux-dev-atlas/src/core/checks/ops.rs")
            || rel.starts_with("crates/bijux-dev-atlas/src/core/checks/ops/")
        {
            continue;
        }
        if content.contains("--bijux-plugin-metadata") || content.contains("--umbrella-version") {
            let owner = rel
                .components()
                .nth(1)
                .and_then(|v| v.as_os_str().to_str())
                .unwrap_or_default()
                .to_string();
            if !owner.is_empty() {
                owners.insert(owner);
            }
        }
    }
    if owners == BTreeSet::from(["bijux-atlas-cli".to_string()]) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_UMBRELLA_DISPATCH_OWNER_INVALID",
            format!("umbrella dispatch ownership must be bijux-atlas-cli only; found {owners:?}"),
            "keep bijux-atlas-cli as the only owner of umbrella dispatch metadata flags",
            Some(Path::new("crates")),
        )])
    }
}

pub(super) fn check_crates_bijux_atlas_help_excludes_dev_commands(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let src = ctx.repo_root.join("crates/bijux-atlas-cli/src/lib.rs");
    let text = fs::read_to_string(&src).map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("Subcommand::Dev") {
        Ok(vec![violation(
            "CRATES_ATLAS_HELP_EXPOSES_DEV_COMMANDS",
            "bijux atlas help surface must not include dev commands".to_string(),
            "move dev command routing under bijux-dev-atlas only",
            Some(Path::new("crates/bijux-atlas-cli/src/lib.rs")),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_crates_bijux_dev_atlas_help_dispatch_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let src = ctx.repo_root.join("crates/bijux-atlas-cli/src/lib.rs");
    let text = fs::read_to_string(&src).map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("bijux dev atlas <command>") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_DEV_ATLAS_DISPATCH_HINT_MISSING",
            "bijux atlas command routing must advertise `bijux dev atlas --help`".to_string(),
            "restore dev atlas dispatch hint in bijux-atlas-cli help routing",
            Some(Path::new("crates/bijux-atlas-cli/src/lib.rs")),
        )])
    }
}

pub(super) fn check_no_string_references_under(
    ctx: &CheckContext<'_>,
    rel_root: &str,
    needle: &str,
    code: &str,
) -> Result<Vec<Violation>, CheckError> {
    let base = ctx.repo_root.join(rel_root);
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&base) {
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if !OPS_TEXT_EXTENSIONS.contains(&ext) && ext != "mk" && ext != "rs" {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        for line in content.lines() {
            if line.contains(needle) {
                let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
                violations.push(violation(
                    code,
                    format!(
                        "forbidden `{needle}` reference in {}: `{}`",
                        rel.display(),
                        line.trim()
                    ),
                    "remove retired references and route through bijux dev atlas",
                    Some(rel),
                ));
                break;
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_no_any_string_references_under(
    ctx: &CheckContext<'_>,
    rel_root: &str,
    needles: &[&str],
    code: &str,
) -> Result<Vec<Violation>, CheckError> {
    let base = ctx.repo_root.join(rel_root);
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&base) {
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if !OPS_TEXT_EXTENSIONS.contains(&ext) && ext != "mk" && ext != "rs" {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        for line in content.lines() {
            for needle in needles {
                if line.contains(needle) {
                    let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
                    violations.push(violation(
                        code,
                        format!(
                            "forbidden `{needle}` reference in {}: `{}`",
                            rel.display(),
                            line.trim()
                        ),
                        "remove direct ops script execution and route through bijux dev atlas",
                        Some(rel),
                    ));
                    break;
                }
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_ops_no_bash_lib_execution(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel in ["makefiles", ".github/workflows"] {
        let root = ctx.repo_root.join(rel);
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            let ext = file
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            if !OPS_TEXT_EXTENSIONS.contains(&ext) && ext != "mk" && ext != "yml" && ext != "yaml" {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('!') && trimmed.contains("rg -n") {
                    continue;
                }
                let invokes_ops_shell = trimmed.contains("bash ops/")
                    || trimmed.contains("source ops/")
                    || trimmed.starts_with(". ops/");
                let sources_retired_lib = trimmed.contains("source ops/_lib")
                    || trimmed.contains(". ops/_lib")
                    || trimmed.contains("bash ops/_lib");
                if invokes_ops_shell || sources_retired_lib {
                    let path = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
                    violations.push(violation(
                        "OPS_BASH_LIB_EXECUTION_REFERENCE_FOUND",
                        format!(
                            "forbidden bash lib execution reference in {}: `{}`",
                            path.display(),
                            trimmed
                        ),
                        "route ops behavior through bijux dev atlas rust commands",
                        Some(path),
                    ));
                    break;
                }
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_removed_system_references_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let needles = [
        "atlasctl",
        "packages/atlasctl",
        "script migration",
        "xtask migration",
        "python era",
        "new control plane",
        "control-plane migration",
    ];
    check_no_any_string_references_under(
        ctx,
        "docs",
        &needles,
        "DOCS_REMOVED_SYSTEM_REFERENCE_FOUND",
    )
}

include!("governance_repo_checks_repo_laws.inc.rs");
