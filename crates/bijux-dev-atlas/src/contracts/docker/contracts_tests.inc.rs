
    fn mk_repo(base: &Path, dockerfile: &str) {
        std::fs::create_dir_all(base.join("docker/images/runtime")).expect("mkdir docker runtime");
        std::fs::write(base.join("docker/images/runtime/Dockerfile"), dockerfile).expect("write dockerfile");
        std::fs::write(base.join("docker/README.md"), "# docker\n").expect("write readme");
        std::fs::write(
            base.join("docker/policy.json"),
            serde_json::json!({
                "schema_version": 1,
                "allow_tagged_images_exceptions": [],
                "required_oci_labels": [
                    "org.opencontainers.image.source",
                    "org.opencontainers.image.version",
                    "org.opencontainers.image.revision",
                    "org.opencontainers.image.created",
                    "org.opencontainers.image.ref.name"
                ]
            })
            .to_string(),
        )
        .expect("write policy");
    }

    #[test]
    fn detects_latest_tag_violation() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "FROM rust:latest\nARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nLABEL org.opencontainers.image.source=\"x\"\nLABEL org.opencontainers.image.version=\"x\"\nLABEL org.opencontainers.image.revision=\"x\"\nLABEL org.opencontainers.image.created=\"x\"\nLABEL org.opencontainers.image.ref.name=\"x\"\n",
        );
        std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", tmp.path().join("Dockerfile")).expect("symlink");
        sync_contract_markdown(tmp.path()).expect("sync contract doc");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                skip_missing_tools: false,
                timeout_seconds: 300,
                fail_fast: false,
                contract_filter: Some("DOCKER-006".to_string()),
                test_filter: Some("docker.from.no_latest".to_string()),
                only_contracts: Vec::new(),
                only_tests: Vec::new(),
                skip_contracts: Vec::new(),
                tags: Vec::new(),
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run contracts");
        assert_eq!(report.fail_count(), 1);
    }

    #[test]
    fn allows_pinned_from_and_required_fields() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "ARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nFROM rust:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa AS builder\nLABEL org.opencontainers.image.source=\"x\"\nLABEL org.opencontainers.image.version=\"x\"\nLABEL org.opencontainers.image.revision=\"x\"\nLABEL org.opencontainers.image.created=\"x\"\nLABEL org.opencontainers.image.ref.name=\"x\"\nCOPY Cargo.toml /workspace/Cargo.toml\n",
        );
        std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\n").expect("write cargo toml");
        std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", tmp.path().join("Dockerfile")).expect("symlink");
        sync_contract_markdown(tmp.path()).expect("sync contract doc");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                skip_missing_tools: false,
                timeout_seconds: 300,
                fail_fast: false,
                contract_filter: None,
                test_filter: None,
                only_contracts: Vec::new(),
                only_tests: Vec::new(),
                skip_contracts: Vec::new(),
                tags: Vec::new(),
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run contracts");
        assert_eq!(report.fail_count(), 0, "report had failures");
    }

    #[test]
    fn parser_handles_multiline_and_preserves_start_line() {
        let text = include_str!("../../../tests/fixtures/dockerfiles/parser_edge_cases.Dockerfile");
        let instructions = parse_dockerfile(text);
        let label = instructions
            .iter()
            .find(|ins| ins.keyword == "LABEL")
            .expect("label instruction");
        assert_eq!(label.line, 7);
        assert!(label.args.contains("org.opencontainers.image.ref.name"));
    }

    #[test]
    fn from_parser_handles_platform_prefix_and_alias() {
        let text = include_str!("../../../tests/fixtures/dockerfiles/parser_edge_cases.Dockerfile");
        let instructions = parse_dockerfile(text);
        let from = instructions
            .iter()
            .find(|ins| ins.keyword == "FROM")
            .expect("from instruction");
        let from_ref = parse_from_ref(&from.args).expect("from ref");
        assert_eq!(
            from_ref,
            "rust:${RUST_VERSION}@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
    }

    #[test]
    fn copy_parser_ignores_copy_from_and_reads_json_array_sources() {
        let text = include_str!("../../../tests/fixtures/dockerfiles/parser_edge_cases.Dockerfile");
        let instructions = parse_dockerfile(text);
        let copy_with_from = instructions
            .iter()
            .find(|ins| ins.keyword == "COPY" && ins.args.contains("--from=builder"))
            .expect("copy --from");
        assert!(extract_copy_sources(&copy_with_from.args).is_empty());

        let copy_json = instructions
            .iter()
            .find(|ins| ins.keyword == "COPY" && ins.args.starts_with('['))
            .expect("json copy");
        assert_eq!(
            extract_copy_sources(&copy_json.args),
            vec!["Cargo.toml".to_string(), "README.md".to_string()]
        );
    }

    #[test]
    fn parser_ignores_comments_and_blank_lines() {
        let instructions = parse_dockerfile("\n# header\n\nARG A=1\n\n# next\nFROM rust:1@sha256:abc\n");
        let keywords = instructions
            .iter()
            .map(|ins| ins.keyword.as_str())
            .collect::<Vec<_>>();
        assert_eq!(keywords, vec!["ARG", "FROM"]);
    }

    #[test]
    fn parser_supports_arg_before_from() {
        let instructions = parse_dockerfile("ARG BASE=rust:1\nFROM ${BASE}@sha256:abc\n");
        assert_eq!(instructions[0].keyword, "ARG");
        assert_eq!(instructions[1].keyword, "FROM");
    }

    #[test]
    fn labels_are_checked_case_insensitively() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "ARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nFROM rust:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nLABEL ORG.OPENCONTAINERS.IMAGE.SOURCE=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.VERSION=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.REVISION=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.CREATED=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.REF.NAME=\"x\"\n",
        );
        std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", tmp.path().join("Dockerfile"))
            .expect("symlink");
        sync_contract_markdown(tmp.path()).expect("sync contract doc");

        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                skip_missing_tools: false,
                timeout_seconds: 300,
                fail_fast: false,
                contract_filter: Some("DOCKER-008".to_string()),
                test_filter: None,
                only_contracts: Vec::new(),
                only_tests: Vec::new(),
                skip_contracts: Vec::new(),
                tags: Vec::new(),
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run contracts");
        assert_eq!(report.fail_count(), 0, "uppercase label keys should pass");
    }

    #[test]
    fn required_image_contract_fails_when_runtime_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join("docker/images/dev")).expect("mkdir image");
        std::fs::write(
            tmp.path().join("docker/images/dev/Dockerfile"),
            "FROM scratch\n",
        )
        .expect("write dockerfile");
        std::fs::write(tmp.path().join("docker/README.md"), "# docker\n").expect("write readme");
        std::fs::write(
            tmp.path().join("docker/policy.json"),
            serde_json::json!({
                "schema_version": 1,
                "required_image_directories": ["runtime"],
                "allowed_image_directories": ["runtime", "dev"],
                "allow_tagged_images_exceptions": [],
                "required_oci_labels": [
                    "org.opencontainers.image.source",
                    "org.opencontainers.image.version",
                    "org.opencontainers.image.revision",
                    "org.opencontainers.image.created",
                    "org.opencontainers.image.ref.name"
                ]
            })
            .to_string(),
        )
        .expect("write policy");
        sync_contract_markdown(tmp.path()).expect("sync contract");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                skip_missing_tools: false,
                timeout_seconds: 300,
                fail_fast: false,
                contract_filter: Some("DOCKER-012".to_string()),
                test_filter: None,
                only_contracts: Vec::new(),
                only_tests: Vec::new(),
                skip_contracts: Vec::new(),
                tags: Vec::new(),
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run");
        assert!(report.fail_count() > 0, "expected missing runtime violation");
    }

    #[test]
    fn forbidden_extra_images_contract_detects_unallowlisted_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "ARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nFROM rust:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nLABEL org.opencontainers.image.source=\"x\"\nLABEL org.opencontainers.image.version=\"x\"\nLABEL org.opencontainers.image.revision=\"x\"\nLABEL org.opencontainers.image.created=\"x\"\nLABEL org.opencontainers.image.ref.name=\"x\"\n",
        );
        std::fs::create_dir_all(tmp.path().join("docker/images/extra")).expect("mkdir extra image");
        std::fs::write(
            tmp.path().join("docker/images/extra/Dockerfile"),
            "FROM scratch\n",
        )
        .expect("write extra dockerfile");
        std::fs::write(
            tmp.path().join("docker/policy.json"),
            serde_json::json!({
                "schema_version": 1,
                "required_image_directories": ["runtime"],
                "allowed_image_directories": ["runtime"],
                "allow_tagged_images_exceptions": [],
                "required_oci_labels": [
                    "org.opencontainers.image.source",
                    "org.opencontainers.image.version",
                    "org.opencontainers.image.revision",
                    "org.opencontainers.image.created",
                    "org.opencontainers.image.ref.name"
                ]
            })
            .to_string(),
        )
        .expect("overwrite policy");
        sync_contract_markdown(tmp.path()).expect("sync contract");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                skip_missing_tools: false,
                timeout_seconds: 300,
                fail_fast: false,
                contract_filter: Some("DOCKER-013".to_string()),
                test_filter: None,
                only_contracts: Vec::new(),
                only_tests: Vec::new(),
                skip_contracts: Vec::new(),
                tags: Vec::new(),
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run");
        assert!(report.fail_count() > 0, "expected forbidden extra image violation");
    }
