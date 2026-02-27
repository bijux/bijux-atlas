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

