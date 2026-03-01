pub(super) fn contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("MAKE-SURFACE-001".to_string()),
            title: "make curated source of truth",
            tests: vec![TestCase {
                id: TestId("make.surface.single_source".to_string()),
                title: "make target list declares root curated targets as its only source",
                kind: TestKind::Pure,
                run: test_make_surface_001_single_source,
            }],
        },
        Contract {
            id: ContractId("MAKE-SURFACE-002".to_string()),
            title: "make curated target budget",
            tests: vec![TestCase {
                id: TestId("make.surface.count_budget".to_string()),
                title: "curated target count stays within the configured max",
                kind: TestKind::Pure,
                run: test_make_surface_002_count_budget,
            }],
        },
        Contract {
            id: ContractId("MAKE-SURFACE-003".to_string()),
            title: "make curated registry sync",
            tests: vec![TestCase {
                id: TestId("make.surface.registry_sync".to_string()),
                title: "curated targets, config registry, and target list stay in sync",
                kind: TestKind::Pure,
                run: test_make_surface_003_registry_sync,
            }],
        },
        Contract {
            id: ContractId("MAKE-SURFACE-004".to_string()),
            title: "make control-plane surface parity",
            tests: vec![TestCase {
                id: TestId("make.surface.control_plane_parity".to_string()),
                title: "the control-plane make surface command stays wired to curated targets",
                kind: TestKind::Pure,
                run: test_make_surface_004_control_plane_parity,
            }],
        },
        Contract {
            id: ContractId("MAKE-SURFACE-005".to_string()),
            title: "make delegate only wrappers",
            tests: vec![TestCase {
                id: TestId("make.surface.delegate_only".to_string()),
                title: "curated targets stay thin and delegate through cargo, make, or dev atlas",
                kind: TestKind::Pure,
                run: test_make_surface_005_delegate_only,
            }],
        },
        Contract {
            id: ContractId("MAKE-INTERNAL-001".to_string()),
            title: "make internal target prefix",
            tests: vec![TestCase {
                id: TestId("make.internal.root_helpers_prefixed".to_string()),
                title: "non-curated root helpers use the _internal- prefix",
                kind: TestKind::Pure,
                run: test_make_internal_001_root_helpers_prefixed,
            }],
        },
        Contract {
            id: ContractId("MAKE-INTERNAL-002".to_string()),
            title: "make scripts banned",
            tests: vec![TestCase {
                id: TestId("make.internal.no_scripts_path".to_string()),
                title: "make sources do not invoke scripts/ directly",
                kind: TestKind::Pure,
                run: |ctx| {
                    test_sources_forbid(
                        ctx,
                        "MAKE-INTERNAL-002",
                        "make.internal.no_scripts_path",
                        "make sources must not invoke scripts paths or inline interpreters directly",
                        &[
                            "scripts/",
                            "tools/",
                            "xtask/",
                            "\tpython ",
                            "\tpython3 ",
                            "\tnode ",
                            "\truby ",
                            "\tperl ",
                            "python -c",
                            "python3 -c",
                            "node -e",
                            "ruby -e",
                            "perl -e",
                        ],
                    )
                },
            }],
        },
        Contract {
            id: ContractId("MAKE-NET-001".to_string()),
            title: "make network commands banned",
            tests: vec![TestCase {
                id: TestId("make.network.no_curl_or_wget".to_string()),
                title: "make sources do not call curl or wget",
                kind: TestKind::Pure,
                run: |ctx| {
                    test_sources_forbid(
                        ctx,
                        "MAKE-NET-001",
                        "make.network.no_curl_or_wget",
                        "make sources must not call curl or wget",
                        &["curl", "wget"],
                    )
                },
            }],
        },
        Contract {
            id: ContractId("MAKE-SHELL-001".to_string()),
            title: "make shell path stability",
            tests: vec![TestCase {
                id: TestId("make.shell.no_cd".to_string()),
                title: "make sources do not depend on cd chains",
                kind: TestKind::Pure,
                run: |ctx| {
                    test_sources_forbid(
                        ctx,
                        "MAKE-SHELL-001",
                        "make.shell.no_cd",
                        "make sources must not use cd in recipes",
                        &["\tcd "],
                    )
                },
            }],
        },
        Contract {
            id: ContractId("MAKE-REPRO-001".to_string()),
            title: "make runenv exports",
            tests: vec![TestCase {
                id: TestId("make.repro.runenv_exports".to_string()),
                title: "run environment exports deterministic run and cache variables",
                kind: TestKind::Pure,
                run: test_make_repro_001_runenv_exports,
            }],
        },
        Contract {
            id: ContractId("MAKE-CI-001".to_string()),
            title: "make workflow curated usage",
            tests: vec![TestCase {
                id: TestId("make.ci.curated_workflow_usage".to_string()),
                title: "workflows call only curated public make targets",
                kind: TestKind::Pure,
                run: test_make_ci_001_curated_workflow_usage,
            }],
        },
        Contract {
            id: ContractId("MAKE-STRUCT-002".to_string()),
            title: "make wrapper files only",
            tests: vec![TestCase {
                id: TestId("make.structure.mk_only".to_string()),
                title: "make contains only allowed modules and registry files",
                kind: TestKind::Pure,
                run: test_make_struct_002_mk_only,
            }],
        },
        Contract {
            id: ContractId("MAKE-STRUCT-010".to_string()),
            title: "make complex recipes dispatch to atlas",
            tests: vec![TestCase {
                id: TestId("make.structure.complex_recipes_dispatch_to_atlas".to_string()),
                title: "recipe-heavy make targets dispatch through the control-plane instead of embedding shell orchestration",
                kind: TestKind::Pure,
                run: test_make_struct_010_complex_recipes_dispatch_to_atlas,
            }],
        },
        Contract {
            id: ContractId("MAKE-OPS-001".to_string()),
            title: "make ops control plane boundary",
            tests: vec![TestCase {
                id: TestId("make.ops.control_plane_only".to_string()),
                title: "ops, k8s, and stack targets route through the ops control plane",
                kind: TestKind::Pure,
                run: test_make_ops_001_ops_targets_use_control_plane,
            }],
        },
        Contract {
            id: ContractId("MAKE-DOCKER-001".to_string()),
            title: "make docker contract boundary",
            tests: vec![TestCase {
                id: TestId("make.docker.contract_runner_only".to_string()),
                title: "docker contract targets route through the docker contracts runner",
                kind: TestKind::Pure,
                run: test_make_docker_001_docker_targets_use_contract_runner,
            }],
        },
        Contract {
            id: ContractId("MAKE-DRIFT-001".to_string()),
            title: "make target list drift",
            tests: vec![TestCase {
                id: TestId("make.surface.target_list_drift".to_string()),
                title: "make target list artifact matches the curated target source",
                kind: TestKind::Pure,
                run: test_make_surface_003_registry_sync,
            }],
        },
        Contract {
            id: ContractId("MAKE-SSOT-001".to_string()),
            title: "make contracts authority",
            tests: vec![TestCase {
                id: TestId("make.ssot.checks_delegate_to_contracts".to_string()),
                title: "make checks delegate contract authority to the Rust contracts runner",
                kind: TestKind::Pure,
                run: test_make_ssot_001_checks_delegate_to_contracts,
            }],
        },
    ]
}

pub(super) fn contract_explain(contract_id: &str) -> Option<&'static str> {
    match contract_id {
        "MAKE-SURFACE-001" => Some("Use make/root.mk:CURATED_TARGETS as the single curated target source."),
        "MAKE-SURFACE-002" => Some("Keep the public make surface within the explicit target budget."),
        "MAKE-SURFACE-003" => Some("Keep the curated target source, target artifact, and config registry synchronized."),
        "MAKE-SURFACE-005" => Some("Curated make targets must stay thin delegates instead of becoming an execution engine."),
        "MAKE-INTERNAL-001" => Some("Non-curated helpers in the root wrapper layer must remain visibly internal."),
        "MAKE-INTERNAL-002" => Some("Make wrappers must not bypass the control plane through direct scripts."),
        "MAKE-NET-001" => Some("Make wrappers must not own network fetches."),
        "MAKE-SHELL-001" => Some("Make wrappers must not depend on directory-changing shell chains."),
        "MAKE-REPRO-001" => Some("Public wrappers must route through exported deterministic run environment defaults."),
        "MAKE-CI-001" => Some("CI workflows may call only the curated make surface."),
        "MAKE-STRUCT-002" => Some("The make/ directory must contain only executable wrapper modules plus its contract docs and target registry."),
        "MAKE-STRUCT-010" => Some("Recipe-heavy make targets must dispatch through `bijux-dev-atlas` instead of embedding orchestration shell logic."),
        "MAKE-OPS-001" => Some("Ops-related make targets must route through `bijux dev atlas ops ...`."),
        "MAKE-DOCKER-001" => Some("Docker-related make targets must route through the docker contract runner."),
        "MAKE-DRIFT-001" => Some("The committed target list artifact must stay aligned with the curated target source."),
        "MAKE-SSOT-001" => Some("Rust contracts, not Make grep rules, are the authority for make contract enforcement."),
        _ => None,
    }
}
