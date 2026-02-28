fn test_root_029_no_nested_git(ctx: &RunContext) -> TestResult {
    fn collect(dir: &std::path::Path, root: &std::path::Path, violations: &mut Vec<Violation>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if path == root.join(".git") || name == ".idea" || name == "artifacts" || name == "target" {
                continue;
            }
            if name == ".git" {
                let rel = path
                    .strip_prefix(root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                violations.push(Violation {
                    contract_id: "ROOT-029".to_string(),
                    test_id: "root.surface.no_nested_git".to_string(),
                    file: Some(rel),
                    line: None,
                    message: "nested git repository is forbidden".to_string(),
                    evidence: None,
                });
                continue;
            }
            collect(&path, root, violations);
        }
    }

    let mut violations = Vec::new();
    collect(&ctx.repo_root, &ctx.repo_root, &mut violations);
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_030_no_vendor_blobs(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-030".to_string(),
                test_id: "root.surface.no_vendor_blobs".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_ascii_lowercase();
        if lower == "vendor" || lower.starts_with("vendor-") || lower.ends_with("-vendor") {
            push_root_violation(
                &mut violations,
                "ROOT-030",
                "root.surface.no_vendor_blobs",
                Some(name),
                "vendor directories and blobs are forbidden at the repo root",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_031_root_file_size_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-031".to_string(),
                test_id: "root.surface.root_file_size_budget".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                push_root_violation(
                    &mut violations,
                    "ROOT-031",
                    "root.surface.root_file_size_budget",
                    Some(name),
                    format!("read metadata failed: {err}"),
                );
                continue;
            }
        };
        if metadata.len() > ROOT_FILE_SIZE_BUDGET_BYTES {
            push_root_violation(
                &mut violations,
                "ROOT-031",
                "root.surface.root_file_size_budget",
                Some(name),
                format!(
                    "root file exceeds size budget: {} > {} bytes",
                    metadata.len(),
                    ROOT_FILE_SIZE_BUDGET_BYTES
                ),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_032_no_nested_toolchain_pins(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for root in ["configs", "ops"] {
        let path = ctx.repo_root.join(root);
        if !path.is_dir() {
            continue;
        }
        let mut found = Vec::new();
        collect_named_files(&path, "rust-toolchain.toml", &mut found);
        collect_named_files(&path, "rust-toolchain", &mut found);
        for item in found {
            let rel = item
                .strip_prefix(&ctx.repo_root)
                .map(|value| value.display().to_string())
                .unwrap_or_else(|_| item.display().to_string());
            push_root_violation(
                &mut violations,
                "ROOT-032",
                "root.surface.no_nested_toolchain_pins",
                Some(rel),
                "nested toolchain authority is forbidden under configs/ or ops/",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_033_release_process_location(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for forbidden in ["RELEASE.md", "RELEASE_PROCESS.md", "RELEASES.md"] {
        if ctx.repo_root.join(forbidden).exists() {
            push_root_violation(
                &mut violations,
                "ROOT-033",
                "root.docs.release_process_location",
                Some(forbidden.to_string()),
                "release process authority must live under docs/ or ops/, not at the repo root",
            );
        }
    }
    if !ctx.repo_root.join("docs/operations").is_dir() && !ctx.repo_root.join("ops").is_dir() {
        push_root_violation(
            &mut violations,
            "ROOT-033",
            "root.docs.release_process_location",
            None,
            "release process authority requires docs/operations/ or ops/ to exist",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_034_single_contract_interface(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for relative in ["README.md", "CONTRIBUTING.md"] {
        let contents = match read_root_text(ctx, relative, "ROOT-034", "root.contracts.single_interface") {
            Ok(contents) => contents,
            Err(result) => return result,
        };
        if contents.contains("cargo run -p bijux-dev-atlas -- contracts") {
            push_root_violation(
                &mut violations,
                "ROOT-034",
                "root.contracts.single_interface",
                Some(relative.to_string()),
                "root docs must not document direct cargo invocation for contracts",
            );
        }
        for line in contents.lines() {
            if line.contains("contracts ") && line.contains("make ") && !line.contains("bijux dev atlas contracts") {
                push_root_violation(
                    &mut violations,
                    "ROOT-034",
                    "root.contracts.single_interface",
                    Some(relative.to_string()),
                    "root docs must route contract command examples through `bijux dev atlas contracts`",
                );
                break;
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_035_make_contract_wrappers_delegate(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(
        ctx,
        "make/checks.mk",
        "ROOT-035",
        "root.make.contract_wrappers_delegate",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    if !contents.contains("contracts make") {
        push_root_violation(
            &mut violations,
            "ROOT-035",
            "root.make.contract_wrappers_delegate",
            Some("make/checks.mk".to_string()),
            "make/checks.mk must delegate through `bijux dev atlas contracts make`",
        );
    }
    if contents.contains("grep ") || contents.contains("rg ") {
        push_root_violation(
            &mut violations,
            "ROOT-035",
            "root.make.contract_wrappers_delegate",
            Some("make/checks.mk".to_string()),
            "make/checks.mk must not reintroduce shell grep-based contract enforcement",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_036_docker_wrappers_delegate(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(
        ctx,
        "make/makefiles/docker.mk",
        "ROOT-036",
        "root.make.docker_wrappers_delegate",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    if !contents.contains("contracts docker --mode static") || !contents.contains("contracts docker --mode effect") {
        push_root_violation(
            &mut violations,
            "ROOT-036",
            "root.make.docker_wrappers_delegate",
            Some("make/makefiles/docker.mk".to_string()),
            "docker make wrappers must delegate both static and effect lanes to `bijux dev atlas contracts docker`",
        );
    }
    if contents.lines().any(|line| line.trim_start().starts_with("@docker ")) {
        push_root_violation(
            &mut violations,
            "ROOT-036",
            "root.make.docker_wrappers_delegate",
            Some("make/makefiles/docker.mk".to_string()),
            "docker make wrappers must not invoke raw docker commands directly",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_037_no_editor_backup_noise(ctx: &RunContext) -> TestResult {
    fn collect(dir: &std::path::Path, root: &std::path::Path, violations: &mut Vec<Violation>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if name == ".git" || name == ".idea" || name == "artifacts" || name == "target" {
                    continue;
                }
                collect(&path, root, violations);
                continue;
            }
            let forbidden = name == ".DS_Store" || name.ends_with(".orig") || name.ends_with('~');
            if forbidden {
                let rel = path
                    .strip_prefix(root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                violations.push(Violation {
                    contract_id: "ROOT-037".to_string(),
                    test_id: "root.surface.no_editor_backup_noise".to_string(),
                    file: Some(rel),
                    line: None,
                    message: "editor backup or platform noise file is forbidden".to_string(),
                    evidence: None,
                });
            }
        }
    }
    let mut violations = Vec::new();
    collect(&ctx.repo_root, &ctx.repo_root, &mut violations);
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_038_gitattributes_line_endings(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join(".gitattributes");
    if !path.exists() {
        return TestResult::Pass;
    }
    let contents = match read_root_text(
        ctx,
        ".gitattributes",
        "ROOT-038",
        "root.gitattributes.line_endings",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    if contents.contains("text=auto") || contents.contains("eol=lf") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-038".to_string(),
            test_id: "root.gitattributes.line_endings".to_string(),
            file: Some(".gitattributes".to_string()),
            line: None,
            message: ".gitattributes must declare text=auto or eol=lf when present".to_string(),
            evidence: None,
        }])
    }
}

