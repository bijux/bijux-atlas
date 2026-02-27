pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("DOCKER-000".to_string()),
            title: "docker directory contract",
            tests: vec![
                TestCase {
                    id: TestId("docker.dir.allowed_markdown".to_string()),
                    title: "only README.md and CONTRACT.md are allowed markdown files",
                    kind: TestKind::Pure,
                    run: test_dir_allowed_markdown,
                },
                TestCase {
                    id: TestId("docker.dir.no_contracts_subdir".to_string()),
                    title: "docker/contracts subdirectory is forbidden",
                    kind: TestKind::Pure,
                    run: test_dir_no_contracts_subdir,
                },
                TestCase {
                    id: TestId("docker.dir.dockerfiles_location".to_string()),
                    title: "Dockerfiles must be under docker/images/**",
                    kind: TestKind::Pure,
                    run: test_dir_dockerfiles_location,
                },
                TestCase {
                    id: TestId("docker.contract_doc.generated_match".to_string()),
                    title: "docker CONTRACT document matches generated registry",
                    kind: TestKind::Pure,
                    run: test_contract_doc_generated_match,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-003".to_string()),
            title: "root Dockerfile policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.root_dockerfile.symlink_or_absent".to_string()),
                    title: "root Dockerfile is symlink or absent",
                    kind: TestKind::Pure,
                    run: test_root_dockerfile_symlink_or_absent,
                },
                TestCase {
                    id: TestId("docker.root_dockerfile.target_runtime".to_string()),
                    title: "root Dockerfile symlink target is runtime Dockerfile",
                    kind: TestKind::Pure,
                    run: test_root_dockerfile_target_runtime,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-004".to_string()),
            title: "dockerfile location policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.dockerfiles.under_images_only".to_string()),
                    title: "Dockerfiles are only under docker/images/**",
                    kind: TestKind::Pure,
                    run: test_dockerfiles_under_images_only,
                },
                TestCase {
                    id: TestId("docker.dockerfiles.filename_convention".to_string()),
                    title: "Dockerfiles follow filename convention",
                    kind: TestKind::Pure,
                    run: test_dockerfiles_filename_convention,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-006".to_string()),
            title: "forbidden tags policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.from.no_latest".to_string()),
                    title: "FROM does not use latest",
                    kind: TestKind::Pure,
                    run: test_from_no_latest,
                },
                TestCase {
                    id: TestId("docker.from.no_floating_tags".to_string()),
                    title: "FROM does not use floating tags unless allowlisted",
                    kind: TestKind::Pure,
                    run: test_from_no_floating_tags,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-007".to_string()),
            title: "digest pinning policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.from.digest_required".to_string()),
                    title: "FROM images require digest pin unless allowlisted",
                    kind: TestKind::Pure,
                    run: test_from_digest_required,
                },
                TestCase {
                    id: TestId("docker.from.repo_digest_format".to_string()),
                    title: "FROM digest format is valid",
                    kind: TestKind::Pure,
                    run: test_from_repo_digest_format,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-008".to_string()),
            title: "required labels policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.labels.required_present".to_string()),
                    title: "required OCI labels are present",
                    kind: TestKind::Pure,
                    run: test_labels_required_present,
                },
                TestCase {
                    id: TestId("docker.labels.required_nonempty".to_string()),
                    title: "required OCI labels are non-empty",
                    kind: TestKind::Pure,
                    run: test_labels_required_nonempty,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-009".to_string()),
            title: "build args defaults policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.args.defaults_present".to_string()),
                    title: "required ARG directives include defaults",
                    kind: TestKind::Pure,
                    run: test_args_defaults_present,
                },
                TestCase {
                    id: TestId("docker.args.required_declared".to_string()),
                    title: "required ARG directives are declared",
                    kind: TestKind::Pure,
                    run: test_args_required_declared,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-010".to_string()),
            title: "forbidden pattern policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.pattern.no_curl_pipe_sh".to_string()),
                    title: "RUN curl|sh is forbidden",
                    kind: TestKind::Pure,
                    run: test_pattern_no_curl_pipe_sh,
                },
                TestCase {
                    id: TestId("docker.pattern.no_add_remote_url".to_string()),
                    title: "ADD remote URL is forbidden",
                    kind: TestKind::Pure,
                    run: test_pattern_no_add_remote_url,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-011".to_string()),
            title: "copy source policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.copy.sources_exist".to_string()),
                    title: "COPY sources must exist",
                    kind: TestKind::Pure,
                    run: test_copy_sources_exist,
                },
                TestCase {
                    id: TestId("docker.copy.no_absolute_sources".to_string()),
                    title: "COPY absolute sources are forbidden",
                    kind: TestKind::Pure,
                    run: test_copy_no_absolute_sources,
                },
                TestCase {
                    id: TestId("docker.copy.no_parent_traversal".to_string()),
                    title: "COPY sources must not use parent traversal",
                    kind: TestKind::Pure,
                    run: test_copy_no_parent_traversal,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-012".to_string()),
            title: "required images exist",
            tests: vec![TestCase {
                id: TestId("docker.images.required_exist".to_string()),
                title: "required image directories include Dockerfile",
                kind: TestKind::Pure,
                run: test_required_images_exist,
            }],
        },
        Contract {
            id: ContractId("DOCKER-013".to_string()),
            title: "forbidden extra images",
            tests: vec![TestCase {
                id: TestId("docker.images.forbidden_extra".to_string()),
                title: "docker image directories are allowlisted",
                kind: TestKind::Pure,
                run: test_forbidden_extra_images,
            }],
        },
        Contract {
            id: ContractId("DOCKER-014".to_string()),
            title: "branch-like tags forbidden",
            tests: vec![TestCase {
                id: TestId("docker.from.no_branch_like_tags".to_string()),
                title: "FROM does not use main, master, edge, or nightly tags",
                kind: TestKind::Pure,
                run: test_from_no_branch_like_tags,
            }],
        },
        Contract {
            id: ContractId("DOCKER-015".to_string()),
            title: "base image allowlist",
            tests: vec![TestCase {
                id: TestId("docker.from.allowlisted_base_images".to_string()),
                title: "FROM images are declared in docker/bases.lock",
                kind: TestKind::Pure,
                run: test_from_images_allowlisted,
            }],
        },
        Contract {
            id: ContractId("DOCKER-016".to_string()),
            title: "base image lock digest",
            tests: vec![TestCase {
                id: TestId("docker.from.digest_matches_lock".to_string()),
                title: "FROM image digests match docker/bases.lock",
                kind: TestKind::Pure,
                run: test_from_digest_matches_bases_lock,
            }],
        },
        Contract {
            id: ContractId("DOCKER-017".to_string()),
            title: "from arg defaults",
            tests: vec![TestCase {
                id: TestId("docker.from.args_have_defaults".to_string()),
                title: "ARG values referenced by FROM have defaults",
                kind: TestKind::Pure,
                run: test_from_args_have_defaults,
            }],
        },
        Contract {
            id: ContractId("DOCKER-018".to_string()),
            title: "from platform override",
            tests: vec![TestCase {
                id: TestId("docker.from.no_platform_override".to_string()),
                title: "FROM does not use --platform unless policy allows it",
                kind: TestKind::Pure,
                run: test_from_no_platform_override,
            }],
        },
        Contract {
            id: ContractId("DOCKER-019".to_string()),
            title: "shell instruction policy",
            tests: vec![TestCase {
                id: TestId("docker.shell.explicit_policy".to_string()),
                title: "Dockerfile SHELL usage follows docker policy",
                kind: TestKind::Pure,
                run: test_shell_policy,
            }],
        },
        Contract {
            id: ContractId("DOCKER-020".to_string()),
            title: "package manager cleanup",
            tests: vec![TestCase {
                id: TestId("docker.run.package_manager_cleanup".to_string()),
                title: "package manager installs include deterministic cleanup",
                kind: TestKind::Pure,
                run: test_package_manager_cleanup,
            }],
        },
        Contract {
            id: ContractId("DOCKER-021".to_string()),
            title: "runtime non-root user",
            tests: vec![TestCase {
                id: TestId("docker.runtime.non_root".to_string()),
                title: "final runtime stage uses a non-root user",
                kind: TestKind::Pure,
                run: test_runtime_non_root,
            }],
        },
        Contract {
            id: ContractId("DOCKER-022".to_string()),
            title: "final stage user declaration",
            tests: vec![TestCase {
                id: TestId("docker.final_stage.user_required".to_string()),
                title: "final stage declares USER explicitly",
                kind: TestKind::Pure,
                run: test_final_stage_has_user,
            }],
        },
        Contract {
            id: ContractId("DOCKER-023".to_string()),
            title: "final stage workdir",
            tests: vec![TestCase {
                id: TestId("docker.final_stage.workdir_required".to_string()),
                title: "final stage declares WORKDIR explicitly",
                kind: TestKind::Pure,
                run: test_final_stage_has_workdir,
            }],
        },
        Contract {
            id: ContractId("DOCKER-024".to_string()),
            title: "final stage process entry",
            tests: vec![TestCase {
                id: TestId("docker.final_stage.entrypoint_or_cmd_required".to_string()),
                title: "final stage declares ENTRYPOINT or CMD",
                kind: TestKind::Pure,
                run: test_final_stage_has_entrypoint_or_cmd,
            }],
        },
        Contract {
            id: ContractId("DOCKER-025".to_string()),
            title: "release labels contract",
            tests: vec![TestCase {
                id: TestId("docker.labels.contract_fields".to_string()),
                title: "release labels include provenance, timestamp, and license fields",
                kind: TestKind::Pure,
                run: test_label_contract_fields,
            }],
        },
        Contract {
            id: ContractId("DOCKER-026".to_string()),
            title: "secret copy guard",
            tests: vec![TestCase {
                id: TestId("docker.copy.no_secrets".to_string()),
                title: "COPY does not include secret-like files",
                kind: TestKind::Pure,
                run: test_copy_no_secrets,
            }],
        },
        Contract {
            id: ContractId("DOCKER-027".to_string()),
            title: "add instruction forbidden",
            tests: vec![TestCase {
                id: TestId("docker.add.forbidden".to_string()),
                title: "Dockerfiles use COPY instead of ADD unless explicitly allowlisted",
                kind: TestKind::Pure,
                run: test_add_forbidden,
            }],
        },
        Contract {
            id: ContractId("DOCKER-028".to_string()),
            title: "multistage build required",
            tests: vec![TestCase {
                id: TestId("docker.build.multistage_required".to_string()),
                title: "builds that compile artifacts use a builder stage",
                kind: TestKind::Pure,
                run: test_compiling_images_are_multistage,
            }],
        },
        Contract {
            id: ContractId("DOCKER-029".to_string()),
            title: "dockerignore required entries",
            tests: vec![TestCase {
                id: TestId("docker.ignore.required_entries".to_string()),
                title: ".dockerignore includes deterministic exclusions",
                kind: TestKind::Pure,
                run: test_dockerignore_required_entries,
            }],
        },
        Contract {
            id: ContractId("DOCKER-030".to_string()),
            title: "reproducible build args",
            tests: vec![TestCase {
                id: TestId("docker.args.repro_build_args".to_string()),
                title: "reproducible build args are declared",
                kind: TestKind::Pure,
                run: test_repro_build_args_present,
            }],
        },
        Contract {
            id: ContractId("DOCKER-031".to_string()),
            title: "final stage network isolation",
            tests: vec![TestCase {
                id: TestId("docker.final_stage.no_network".to_string()),
                title: "final stage does not fetch over the network",
                kind: TestKind::Pure,
                run: test_no_network_in_final_stage,
            }],
        },
        Contract {
            id: ContractId("DOCKER-032".to_string()),
            title: "final stage package manager isolation",
            tests: vec![TestCase {
                id: TestId("docker.final_stage.no_package_manager".to_string()),
                title: "final stage does not run package managers",
                kind: TestKind::Pure,
                run: test_no_package_manager_in_final_stage,
            }],
        },
        Contract {
            id: ContractId("DOCKER-033".to_string()),
            title: "image smoke manifest",
            tests: vec![TestCase {
                id: TestId("docker.images.smoke_manifest".to_string()),
                title: "each Docker image is listed with a smoke command in docker/images.manifest.json",
                kind: TestKind::Pure,
                run: test_images_have_smoke_manifest,
            }],
        },
        Contract {
            id: ContractId("DOCKER-100".to_string()),
            title: "build succeeds",
            tests: vec![TestCase {
                id: TestId("docker.build.runtime_image".to_string()),
                title: "runtime image build succeeds",
                kind: TestKind::Subprocess,
                run: test_effect_build_runtime_image,
            }],
        },
        Contract {
            id: ContractId("DOCKER-101".to_string()),
            title: "runtime smoke checks",
            tests: vec![
                TestCase {
                    id: TestId("docker.smoke.version".to_string()),
                    title: "runtime image prints version",
                    kind: TestKind::Subprocess,
                    run: test_effect_smoke_version,
                },
                TestCase {
                    id: TestId("docker.smoke.help".to_string()),
                    title: "runtime image prints help",
                    kind: TestKind::Subprocess,
                    run: test_effect_smoke_help,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-102".to_string()),
            title: "sbom generated",
            tests: vec![TestCase {
                id: TestId("docker.sbom.generated".to_string()),
                title: "syft generates a JSON SBOM",
                kind: TestKind::Subprocess,
                run: test_effect_sbom_generated,
            }],
        },
        Contract {
            id: ContractId("DOCKER-103".to_string()),
            title: "scan passes policy",
            tests: vec![TestCase {
                id: TestId("docker.scan.severity_threshold".to_string()),
                title: "trivy scan passes configured severity threshold",
                kind: TestKind::Network,
                run: test_effect_scan_passes_policy,
            }],
        },
    ])
}
