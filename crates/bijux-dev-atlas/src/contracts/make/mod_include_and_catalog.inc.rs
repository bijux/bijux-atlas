fn test_make_env_002_role_boundary(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-ENV-002";
    let test_id = "make.env.role_boundary";
    let macros_path = ctx.repo_root.join("make/macros.mk");
    let runenv_path = ctx.repo_root.join("make/runenv.mk");
    let macros_text = match std::fs::read_to_string(&macros_path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&macros_path, &ctx.repo_root)),
                line: Some(1),
                message: format!("read {} failed: {err}", macros_path.display()),
                evidence: None,
            }]);
        }
    };
    let runenv_text = match std::fs::read_to_string(&runenv_path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&runenv_path, &ctx.repo_root)),
                line: Some(1),
                message: format!("read {} failed: {err}", runenv_path.display()),
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    for line in macros_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("export ")
            || trimmed.starts_with("include ")
            || trimmed
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            violations.push(violation(
                contract_id,
                test_id,
                &macros_path,
                &ctx.repo_root,
                "make/macros.mk must contain only pure macro helpers",
            ));
            break;
        }
    }
    let has_export = runenv_text
        .lines()
        .any(|line| line.trim_start().starts_with("export "));
    if !has_export {
        violations.push(violation(
            contract_id,
            test_id,
            &runenv_path,
            &ctx.repo_root,
            "make/runenv.mk must export deterministic environment defaults",
        ));
    }
    for line in runenv_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("export ") {
            continue;
        }
        if trimmed.contains(" =") || trimmed.contains("= ") || trimmed.contains(" = ") {
            let name = trimmed.split('=').next().unwrap_or("").trim();
            if name
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_lowercase())
            {
                violations.push(violation(
                    contract_id,
                    test_id,
                    &runenv_path,
                    &ctx.repo_root,
                    "make/runenv.mk must not define helper macros",
                ));
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

fn test_make_include_001_root_single_entrypoint(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-INCLUDE-001";
    let test_id = "make.includes.root_single_entrypoint";
    let makefile = ctx.repo_root.join("Makefile");
    let includes = match include_lines(&makefile) {
        Ok(lines) => lines,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("Makefile".to_string()),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    if includes == ["make/public.mk".to_string()] {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            &makefile,
            &ctx.repo_root,
            "Makefile must include exactly one file: make/public.mk",
        )])
    }
}

fn test_make_include_002_public_surface(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-INCLUDE-002";
    let test_id = "make.includes.public_surface";
    let public_path = ctx.repo_root.join("make/public.mk");
    let internal_path = ctx.repo_root.join("make/_internal.mk");
    let includes = match include_lines(&public_path) {
        Ok(lines) => lines,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&public_path, &ctx.repo_root)),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    let expected = vec![
        "make/_internal.mk".to_string(),
        "make/checks.mk".to_string(),
        "make/contracts.mk".to_string(),
        "make/macros.mk".to_string(),
        "make/paths.mk".to_string(),
        "make/vars.mk".to_string(),
    ];
    let internal_includes = match include_lines(&internal_path) {
        Ok(lines) => lines,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&internal_path, &ctx.repo_root)),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    if includes != expected {
        violations.push(violation(
            contract_id,
            test_id,
            &public_path,
            &ctx.repo_root,
            "make/public.mk must include only vars, paths, macros, _internal, and checks",
        ));
    }
    if internal_includes != ["make/root.mk".to_string()] {
        violations.push(violation(
            contract_id,
            test_id,
            &internal_path,
            &ctx.repo_root,
            "make/_internal.mk must include exactly one file: make/root.mk",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_include_003_acyclic(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-INCLUDE-003";
    let test_id = "make.includes.acyclic";
    let files = sorted_make_sources(&ctx.repo_root);
    let mut edges = BTreeMap::<String, Vec<String>>::new();
    let mut violations = Vec::new();
    for path in &files {
        let rel_path = rel(path, &ctx.repo_root);
        let includes = match include_lines(path) {
            Ok(lines) => lines,
            Err(err) => {
                violations.push(Violation {
                    contract_id: contract_id.to_string(),
                    test_id: test_id.to_string(),
                    file: Some(rel_path.clone()),
                    line: Some(1),
                    message: err,
                    evidence: None,
                });
                continue;
            }
        };
        edges.insert(rel_path, includes);
    }
    fn visit(
        node: &str,
        edges: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> Option<String> {
        if visited.contains(node) {
            return None;
        }
        if !visiting.insert(node.to_string()) {
            return Some(node.to_string());
        }
        for next in edges.get(node).into_iter().flatten() {
            if edges.contains_key(next) {
                if let Some(cycle) = visit(next, edges, visiting, visited) {
                    return Some(cycle);
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
        None
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for node in edges.keys() {
        if let Some(cycle) = visit(node, &edges, &mut visiting, &mut visited) {
            let path = ctx.repo_root.join(&cycle);
            violations.push(violation(
                contract_id,
                test_id,
                &path,
                &ctx.repo_root,
                "make include graph must be acyclic",
            ));
            break;
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    let mut contracts = vec![
        Contract {
            id: ContractId("MAKE-DIR-001".to_string()),
            title: "make root docs boundary",
            tests: vec![TestCase {
                id: TestId("make.docs.allowed_root_docs_only".to_string()),
                title: "make root keeps only README.md and CONTRACT.md as markdown",
                kind: TestKind::Pure,
                run: test_make_dir_001_allowed_root_docs_only,
            }],
        },
        Contract {
            id: ContractId("MAKE-DIR-002".to_string()),
            title: "make nested docs removal",
            tests: vec![TestCase {
                id: TestId("make.docs.no_nested_markdown".to_string()),
                title: "make contains no nested markdown",
                kind: TestKind::Pure,
                run: test_make_dir_002_no_nested_markdown,
            }],
        },
        Contract {
            id: ContractId("MAKE-DIR-003".to_string()),
            title: "make root file boundary",
            tests: vec![TestCase {
                id: TestId("make.surface.allowed_root_files".to_string()),
                title: "make root contains only curated wrapper files",
                kind: TestKind::Pure,
                run: test_make_dir_003_allowed_root_files,
            }],
        },
        Contract {
            id: ContractId("MAKE-STRUCT-001".to_string()),
            title: "root Makefile stays minimal",
            tests: vec![TestCase {
                id: TestId("make.structure.root_makefile_minimal".to_string()),
                title: "Makefile stays <=200 lines and includes only make/public.mk",
                kind: TestKind::Pure,
                run: test_make_struct_001_root_makefile_minimal,
            }],
        },
        Contract {
            id: ContractId("MAKE-STRUCT-003".to_string()),
            title: "make modules stay on the approved whitelist",
            tests: vec![TestCase {
                id: TestId("make.structure.allowed_modules".to_string()),
                title: "top-level make modules stay within the approved whitelist",
                kind: TestKind::Pure,
                run: test_make_struct_003_allowed_modules,
            }],
        },
        Contract {
            id: ContractId("MAKE-STRUCT-004".to_string()),
            title: "make modules declare headers",
            tests: vec![TestCase {
                id: TestId("make.structure.module_headers".to_string()),
                title: "top-level make modules declare scope and public-target headers",
                kind: TestKind::Pure,
                run: test_make_struct_004_module_headers,
            }],
        },
        Contract {
            id: ContractId("MAKE-STRUCT-007".to_string()),
            title: "make help surface remains sorted",
            tests: vec![TestCase {
                id: TestId("make.structure.help_surface_sorted".to_string()),
                title: "CURATED_TARGETS stays alphabetically sorted",
                kind: TestKind::Pure,
                run: test_make_struct_007_help_surface_sorted,
            }],
        },
        Contract {
            id: ContractId("MAKE-ENV-001".to_string()),
            title: "make env file singularity",
            tests: vec![TestCase {
                id: TestId("make.env.single_macros_and_runenv".to_string()),
                title: "make keeps one macros file and one run-environment file",
                kind: TestKind::Pure,
                run: test_make_env_001_single_macros_and_runenv,
            }],
        },
        Contract {
            id: ContractId("MAKE-ENV-002".to_string()),
            title: "make env role boundary",
            tests: vec![TestCase {
                id: TestId("make.env.role_boundary".to_string()),
                title: "macros and run-environment files keep distinct responsibilities",
                kind: TestKind::Pure,
                run: test_make_env_002_role_boundary,
            }],
        },
        Contract {
            id: ContractId("MAKE-INCLUDE-001".to_string()),
            title: "make root include entrypoint",
            tests: vec![TestCase {
                id: TestId("make.includes.root_single_entrypoint".to_string()),
                title: "Makefile includes only make/public.mk",
                kind: TestKind::Pure,
                run: test_make_include_001_root_single_entrypoint,
            }],
        },
        Contract {
            id: ContractId("MAKE-INCLUDE-002".to_string()),
            title: "make public include boundary",
            tests: vec![TestCase {
                id: TestId("make.includes.public_surface".to_string()),
                title: "make public entrypoint includes only the approved wrapper modules",
                kind: TestKind::Pure,
                run: test_make_include_002_public_surface,
            }],
        },
        Contract {
            id: ContractId("MAKE-INCLUDE-003".to_string()),
            title: "make include graph acyclic",
            tests: vec![TestCase {
                id: TestId("make.includes.acyclic".to_string()),
                title: "make include graph is acyclic",
                kind: TestKind::Pure,
                run: test_make_include_003_acyclic,
            }],
        },
        Contract {
            id: ContractId("MAKE-001".to_string()),
            title: "contracts gate uses make/contracts.mk as single entrypoint",
            tests: vec![TestCase {
                id: TestId("make.contracts.single_entrypoint".to_string()),
                title: "contracts targets are sourced from make/contracts.mk via make/public.mk",
                kind: TestKind::Pure,
                run: test_make_contracts_001_single_entrypoint,
            }],
        },
        Contract {
            id: ContractId("MAKE-002".to_string()),
            title: "contracts gate public targets are explicit and stable",
            tests: vec![TestCase {
                id: TestId("make.contracts.target_surface".to_string()),
                title: "contracts.mk declares only the approved contracts targets",
                kind: TestKind::Pure,
                run: test_make_contracts_002_target_surface,
            }],
        },
        Contract {
            id: ContractId("MAKE-003".to_string()),
            title: "contracts gate targets are thin delegates to the contracts runner",
            tests: vec![TestCase {
                id: TestId("make.contracts.delegate_only".to_string()),
                title: "contracts.mk delegates via bijux-dev-atlas contracts invocations only",
                kind: TestKind::Pure,
                run: test_make_contracts_003_delegate_only,
            }],
        },
    ];
    contracts.extend(surface_contracts::contracts());
    contracts.extend(wrapper_contracts::contracts());
    Ok(contracts)
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "MAKE-DIR-001" => {
            "Keep make markdown authority limited to make/README.md and make/CONTRACT.md.".to_string()
        }
        "MAKE-DIR-002" => {
            "Remove nested markdown from make/ so implementation and policy do not drift."
                .to_string()
        }
        "MAKE-DIR-003" => {
            "Freeze the top-level make/ surface to curated wrapper files only.".to_string()
        }
        "MAKE-STRUCT-001" => {
            "Keep the root Makefile minimal so all logic stays in reviewed make modules.".to_string()
        }
        "MAKE-STRUCT-003" => {
            "Restrict top-level make modules to the approved whitelist so the wrapper surface does not sprawl.".to_string()
        }
        "MAKE-STRUCT-004" => {
            "Every make module must declare its scope and public-target surface near the top.".to_string()
        }
        "MAKE-STRUCT-007" => {
            "Keep CURATED_TARGETS alphabetically sorted so `make help` and generated target registries stay stable.".to_string()
        }
        "MAKE-SURFACE-004" => {
            "The control plane must expose the same curated make surface that root make wrappers publish.".to_string()
        }
        "MAKE-ENV-001" => {
            "Keep one macros source and one run-environment source to avoid env drift.".to_string()
        }
        "MAKE-ENV-002" => {
            "Separate pure macros from exported runtime defaults so responsibility stays obvious."
                .to_string()
        }
        "MAKE-INCLUDE-001" => {
            "Makefile must route through one public entrypoint instead of accumulating direct includes."
                .to_string()
        }
        "MAKE-INCLUDE-002" => {
            "The public make entrypoint must include only the approved wrapper modules.".to_string()
        }
        "MAKE-INCLUDE-003" => {
            "Keep the make include graph acyclic so wrapper composition stays reviewable.".to_string()
        }
        "MAKE-001" => "Define contracts gate targets in make/contracts.mk and include them through make/public.mk.".to_string(),
        "MAKE-002" => "Keep contracts public target surface explicit and stable so gate usage is predictable.".to_string(),
        "MAKE-003" => "Contracts make targets must remain thin delegates to bijux-dev-atlas contracts commands.".to_string(),
        _ => surface_contracts::contract_explain(contract_id)
            .or_else(|| wrapper_contracts::contract_explain(contract_id))
            .unwrap_or("Unknown make contract id.")
            .to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts make --mode static"
}
