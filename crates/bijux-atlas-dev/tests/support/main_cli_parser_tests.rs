// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use clap::Parser;
    use std::fs;
    use std::path::PathBuf;

    fn test_fixture_root(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn ops_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "ops", "list"],
            vec!["bijux-atlas-dev", "ops", "explain", "render"],
            vec!["bijux-atlas-dev", "ops", "doctor"],
            vec!["bijux-atlas-dev", "ops", "validate"],
            vec!["bijux-atlas-dev", "ops", "inventory"],
            vec!["bijux-atlas-dev", "ops", "docs"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "conformance",
                "--allow-subprocess",
            ],
            vec!["bijux-atlas-dev", "ops", "report", "--allow-write"],
            vec!["bijux-atlas-dev", "ops", "render", "--allow-subprocess"],
            vec!["bijux-atlas-dev", "ops", "install", "--plan"],
            vec!["bijux-atlas-dev", "ops", "status"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "status",
                "--target",
                "k8s",
                "--allow-subprocess",
            ],
            vec!["bijux-atlas-dev", "ops", "list-profiles"],
            vec!["bijux-atlas-dev", "ops", "explain-profile", "kind"],
            vec!["bijux-atlas-dev", "ops", "list-tools", "--allow-subprocess"],
            vec!["bijux-atlas-dev", "ops", "tools", "list"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "tools",
                "verify",
                "--allow-subprocess",
            ],
            vec!["bijux-atlas-dev", "ops", "tools", "doctor"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "verify-tools",
                "--allow-subprocess",
            ],
            vec!["bijux-atlas-dev", "ops", "list-actions"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "up",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "down",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec!["bijux-atlas-dev", "ops", "clean"],
            vec!["bijux-atlas-dev", "ops", "cleanup"],
            vec!["bijux-atlas-dev", "ops", "stack", "plan"],
            vec!["bijux-atlas-dev", "ops", "stack", "status"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "stack",
                "up",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "stack",
                "down",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "stack",
                "reset",
                "--reset-run-id",
                "ops_reset",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "plan",
                "--run-id",
                "ops_render_kind_golden",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "apply",
                "--apply",
                "--allow-subprocess",
                "--allow-write",
                "--run-id",
                "ops_render_kind_golden",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "dry-run",
                "--allow-subprocess",
                "--allow-write",
                "--run-id",
                "ops_render_kind_golden",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "conformance",
                "--allow-subprocess",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "wait",
                "--allow-subprocess",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "logs",
                "--allow-subprocess",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "ports",
                "--allow-subprocess",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "port-forward",
                "--allow-subprocess",
                "--allow-network",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "render",
                "--allow-subprocess",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "test",
                "--allow-subprocess",
            ],
            vec![
                "bijux-atlas-dev",
                "ops",
                "k8s",
                "status",
                "--target",
                "pods",
                "--allow-subprocess",
            ],
            vec!["bijux-atlas-dev", "ops", "load", "plan", "mixed"],
            vec!["bijux-atlas-dev", "ops", "load", "run", "mixed"],
            vec!["bijux-atlas-dev", "ops", "load", "report", "mixed"],
            vec!["bijux-atlas-dev", "ops", "e2e", "run"],
            vec!["bijux-atlas-dev", "ops", "obs", "drill", "run"],
            vec!["bijux-atlas-dev", "ops", "obs", "verify"],
            vec!["bijux-atlas-dev", "ops", "suite", "list"],
            vec!["bijux-atlas-dev", "ops", "suite", "run", "k8s"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "reset",
                "--reset-run-id",
                "ops_reset",
            ],
            vec!["bijux-atlas-dev", "ops", "pins", "check"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "pins",
                "update",
                "--allow-subprocess",
                "--i-know-what-im-doing",
            ],
            vec!["bijux-atlas-dev", "ops", "generate", "pins-index"],
            vec![
                "bijux-atlas-dev",
                "ops",
                "generate",
                "pins-index",
                "--check",
            ],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Ops { .. }) => {}
                _ => panic!("expected ops command"),
            }
        }
    }

    #[test]
    fn check_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "registry", "doctor"],
            vec!["bijux-atlas-dev", "list"],
            vec!["bijux-atlas-dev", "list", "--format", "json"],
            vec!["bijux-atlas-dev", "describe", "checks_ops_surface_manifest"],
            vec!["bijux-atlas-dev", "registry", "status"],
            vec!["bijux-atlas-dev", "run", "checks_ops_surface_manifest"],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(
                    crate::cli::Command::Registry { .. }
                    | crate::cli::Command::List { .. }
                    | crate::cli::Command::Describe { .. }
                    | crate::cli::Command::Run { .. },
                ) => {}
                _ => panic!("expected registry/list/describe/run command"),
            }
        }
    }

    #[test]
    fn checks_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "suites", "list"],
            vec![
                "bijux-atlas-dev",
                "suites",
                "describe",
                "--suite",
                "deep",
                "--format",
                "json",
            ],
            vec!["bijux-atlas-dev", "suites", "run", "--suite", "ci_fast"],
            vec!["bijux-atlas-dev", "suites", "lint"],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Suites { .. }) => {}
                _ => panic!("expected suites command"),
            }
        }
    }

    #[test]
    fn release_subcommands_parse() {
        let cli =
            crate::Cli::try_parse_from(vec!["bijux-atlas-dev", "release", "check"]).expect("parse");
        match cli.command {
            Some(crate::cli::Command::Release { .. }) => {}
            _ => panic!("expected release command"),
        }
    }

    #[test]
    fn top_level_version_and_help_inventory_parse() {
        for argv in [
            vec!["bijux-atlas-dev", "version"],
            vec!["bijux-atlas-dev", "version", "--format", "json"],
            vec!["bijux-atlas-dev", "help"],
            vec!["bijux-atlas-dev", "help", "--format", "json"],
            vec!["bijux-atlas-dev", "--print-boundaries"],
        ] {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Version { .. })
                | Some(crate::cli::Command::Help { .. }) => {}
                None => {}
                _ => panic!("expected top-level version/help command"),
            }
        }
    }

    #[test]
    fn docs_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "docs", "doctor"],
            vec!["bijux-atlas-dev", "docs", "deploy-plan"],
            vec!["bijux-atlas-dev", "docs", "pages-smoke", "--allow-network"],
            vec!["bijux-atlas-dev", "docs", "where"],
            vec!["bijux-atlas-dev", "docs", "spine", "validate"],
            vec!["bijux-atlas-dev", "docs", "spine", "report"],
            vec!["bijux-atlas-dev", "docs", "merge", "validate"],
            vec!["bijux-atlas-dev", "docs", "check", "--allow-subprocess"],
            vec!["bijux-atlas-dev", "docs", "validate"],
            vec!["bijux-atlas-dev", "docs", "clean", "--allow-write"],
            vec!["bijux-atlas-dev", "docs", "lint"],
            vec!["bijux-atlas-dev", "docs", "links"],
            vec!["bijux-atlas-dev", "docs", "inventory"],
            vec!["bijux-atlas-dev", "docs", "graph"],
            vec!["bijux-atlas-dev", "docs", "top", "--limit", "20"],
            vec!["bijux-atlas-dev", "docs", "dead"],
            vec!["bijux-atlas-dev", "docs", "duplicates"],
            vec!["bijux-atlas-dev", "docs", "grep", "bijux dev atlas"],
            vec![
                "bijux-atlas-dev",
                "docs",
                "build",
                "--allow-subprocess",
                "--allow-write",
                "--strict",
            ],
            vec![
                "bijux-atlas-dev",
                "docs",
                "serve",
                "--allow-subprocess",
                "--allow-network",
                "--include-drafts",
            ],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Docs { .. }) => {}
                _ => panic!("expected docs command"),
            }
        }
    }

    #[test]
    fn mkdocs_nav_parser_extracts_refs() {
        let root = test_fixture_root("docs-mini");
        let refs = crate::mkdocs_nav_refs(&root).expect("mkdocs nav");
        let paths = refs.into_iter().map(|(_, p)| p).collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec![
                "reference/commands.md".to_string(),
                "reference/configs.md".to_string(),
                "index.md".to_string(),
                "sub/intro.md".to_string(),
                "reference/make-targets.md".to_string(),
                "reference/schemas.md".to_string(),
            ]
        );
    }

    #[test]
    fn docs_link_resolver_accepts_fixture_links() {
        let repo_root = test_fixture_root("docs-mini");
        let ctx = crate::DocsContext {
            docs_root: repo_root.join("docs"),
            artifacts_root: repo_root.join("artifacts"),
            run_id: crate::RunId::from_seed("docs_fixture"),
            repo_root: repo_root.clone(),
        };
        let common = crate::cli::DocsCommonArgs {
            repo_root: Some(repo_root),
            artifacts_root: None,
            run_id: None,
            format: crate::cli::FormatArg::Json,
            out: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            strict: false,
            include_drafts: false,
        };
        let payload = crate::docs_links_payload(&ctx, &common).expect("links payload");
        assert_eq!(
            payload
                .get("errors")
                .and_then(|v| v.as_array())
                .map(|v| v.len()),
            Some(0)
        );
        assert_eq!(
            payload
                .get("external_link_check")
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn configs_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "configs", "doctor"],
            vec!["bijux-atlas-dev", "configs", "print"],
            vec!["bijux-atlas-dev", "configs", "list"],
            vec!["bijux-atlas-dev", "configs", "verify"],
            vec!["bijux-atlas-dev", "configs", "validate", "--strict"],
            vec!["bijux-atlas-dev", "configs", "lint"],
            vec!["bijux-atlas-dev", "configs", "fmt", "--check"],
            vec!["bijux-atlas-dev", "configs", "inventory"],
            vec!["bijux-atlas-dev", "configs", "compile", "--allow-write"],
            vec!["bijux-atlas-dev", "configs", "diff"],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Configs { .. }) => {}
                _ => panic!("expected configs command"),
            }
        }
    }

    #[test]
    fn docker_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "docker", "build", "--allow-subprocess"],
            vec!["bijux-atlas-dev", "docker", "check", "--allow-subprocess"],
            vec!["bijux-atlas-dev", "docker", "smoke", "--allow-subprocess"],
            vec![
                "bijux-atlas-dev",
                "docker",
                "scan",
                "--allow-subprocess",
                "--allow-network",
            ],
            vec!["bijux-atlas-dev", "docker", "sbom", "--allow-subprocess"],
            vec!["bijux-atlas-dev", "docker", "policy", "check"],
            vec!["bijux-atlas-dev", "docker", "lock", "--allow-write"],
            vec![
                "bijux-atlas-dev",
                "docker",
                "push",
                "--allow-subprocess",
                "--allow-network",
                "--i-know-what-im-doing",
            ],
            vec![
                "bijux-atlas-dev",
                "docker",
                "release",
                "--allow-subprocess",
                "--allow-network",
                "--i-know-what-im-doing",
            ],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Docker { .. }) => {}
                _ => panic!("expected docker command"),
            }
        }
    }

    #[test]
    fn system_cluster_commands_require_explicit_runtime_configs() {
        let missing_configs =
            crate::Cli::try_parse_from(vec!["bijux-atlas-dev", "system", "cluster", "topology"]);
        assert!(
            missing_configs.is_err(),
            "system cluster topology should require explicit runtime config paths"
        );

        let cli = crate::Cli::try_parse_from(vec![
            "bijux-atlas-dev",
            "system",
            "cluster",
            "topology",
            "--cluster-config",
            "configs/examples/operations/runtime/cluster-config.json",
            "--node-config",
            "configs/examples/operations/runtime/node-config.json",
        ])
        .expect("parse");
        match cli.command {
            Some(crate::cli::Command::System { .. }) => {}
            _ => panic!("expected system command"),
        }
    }

    #[test]
    fn build_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "build", "bin"],
            vec![
                "bijux-atlas-dev",
                "build",
                "dist",
                "--allow-subprocess",
                "--allow-write",
            ],
            vec!["bijux-atlas-dev", "build", "doctor", "--format", "json"],
            vec!["bijux-atlas-dev", "build", "clean", "--allow-write"],
            vec![
                "bijux-atlas-dev",
                "build",
                "clean",
                "--allow-write",
                "--include-bin",
            ],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Build { .. }) => {}
                _ => panic!("expected build command"),
            }
        }
    }
    #[test]
    fn policies_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "policies", "list"],
            vec!["bijux-atlas-dev", "policies", "explain", "repo"],
            vec!["bijux-atlas-dev", "policies", "report"],
            vec!["bijux-atlas-dev", "policies", "print"],
            vec![
                "bijux-atlas-dev",
                "policies",
                "validate",
                "--format",
                "json",
            ],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Policies { .. }) => {}
                _ => panic!("expected policies command"),
            }
        }
    }

    #[test]
    fn workflows_subcommands_parse() {
        let commands = [vec!["bijux-atlas-dev", "workflows", "validate"]];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Workflows { .. }) => {}
                _ => panic!("expected workflows command"),
            }
        }
    }

    #[test]
    fn gates_subcommands_parse() {
        let commands = [
            vec!["bijux-atlas-dev", "gates", "list"],
            vec!["bijux-atlas-dev", "gates", "run"],
            vec!["bijux-atlas-dev", "gates", "run", "--suite", "deep"],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Gates { .. }) => {}
                _ => panic!("expected gates command"),
            }
        }
    }

    #[test]
    fn parse_config_file_supports_json_yaml_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let json = dir.path().join("a.json");
        let yaml = dir.path().join("b.yaml");
        let toml = dir.path().join("c.toml");
        fs::write(&json, "{\"x\":1}").expect("json");
        fs::write(&yaml, "x: 1\n").expect("yaml");
        fs::write(&toml, "x = 1\n").expect("toml");
        assert!(crate::parse_config_file(&json).is_ok());
        assert!(crate::parse_config_file(&yaml).is_ok());
        assert!(crate::parse_config_file(&toml).is_ok());
    }
}
