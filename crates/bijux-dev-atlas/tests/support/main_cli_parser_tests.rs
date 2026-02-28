// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use clap::Parser;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn ops_subcommands_parse() {
        let commands = [
            vec!["bijux-dev-atlas", "ops", "list"],
            vec!["bijux-dev-atlas", "ops", "explain", "render"],
            vec!["bijux-dev-atlas", "ops", "doctor"],
            vec!["bijux-dev-atlas", "ops", "validate"],
            vec!["bijux-dev-atlas", "ops", "inventory"],
            vec!["bijux-dev-atlas", "ops", "docs"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "conformance",
                "--allow-subprocess",
            ],
            vec!["bijux-dev-atlas", "ops", "report", "--allow-write"],
            vec!["bijux-dev-atlas", "ops", "render", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "install", "--plan"],
            vec!["bijux-dev-atlas", "ops", "status"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "status",
                "--target",
                "k8s",
                "--allow-subprocess",
            ],
            vec!["bijux-dev-atlas", "ops", "list-profiles"],
            vec!["bijux-dev-atlas", "ops", "explain-profile", "kind"],
            vec!["bijux-dev-atlas", "ops", "list-tools", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "tools", "list"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "tools",
                "verify",
                "--allow-subprocess",
            ],
            vec!["bijux-dev-atlas", "ops", "tools", "doctor"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "verify-tools",
                "--allow-subprocess",
            ],
            vec!["bijux-dev-atlas", "ops", "list-actions"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "up",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "down",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec!["bijux-dev-atlas", "ops", "clean"],
            vec!["bijux-dev-atlas", "ops", "cleanup"],
            vec!["bijux-dev-atlas", "ops", "stack", "plan"],
            vec!["bijux-dev-atlas", "ops", "stack", "status"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "stack",
                "up",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "stack",
                "down",
                "--allow-subprocess",
                "--allow-write",
                "--allow-network",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "stack",
                "reset",
                "--reset-run-id",
                "ops_reset",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "plan",
                "--run-id",
                "ops_render_kind_golden",
            ],
            vec![
                "bijux-dev-atlas",
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
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "dry-run",
                "--allow-subprocess",
                "--allow-write",
                "--run-id",
                "ops_render_kind_golden",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "conformance",
                "--allow-subprocess",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "wait",
                "--allow-subprocess",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "logs",
                "--allow-subprocess",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "ports",
                "--allow-subprocess",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "port-forward",
                "--allow-subprocess",
                "--allow-network",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "render",
                "--allow-subprocess",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "test",
                "--allow-subprocess",
            ],
            vec![
                "bijux-dev-atlas",
                "ops",
                "k8s",
                "status",
                "--target",
                "pods",
                "--allow-subprocess",
            ],
            vec!["bijux-dev-atlas", "ops", "load", "plan", "mixed"],
            vec!["bijux-dev-atlas", "ops", "load", "run", "mixed"],
            vec!["bijux-dev-atlas", "ops", "load", "report", "mixed"],
            vec!["bijux-dev-atlas", "ops", "e2e", "run"],
            vec!["bijux-dev-atlas", "ops", "obs", "drill", "run"],
            vec!["bijux-dev-atlas", "ops", "obs", "verify"],
            vec!["bijux-dev-atlas", "ops", "suite", "list"],
            vec!["bijux-dev-atlas", "ops", "suite", "run", "k8s"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "reset",
                "--reset-run-id",
                "ops_reset",
            ],
            vec!["bijux-dev-atlas", "ops", "pins", "check"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "pins",
                "update",
                "--allow-subprocess",
                "--i-know-what-im-doing",
            ],
            vec!["bijux-dev-atlas", "ops", "generate", "pins-index"],
            vec![
                "bijux-dev-atlas",
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
            vec!["bijux-dev-atlas", "check", "registry", "doctor"],
            vec!["bijux-dev-atlas", "check", "list"],
            vec!["bijux-dev-atlas", "check", "list", "--json"],
            vec![
                "bijux-dev-atlas",
                "check",
                "explain",
                "checks_ops_surface_manifest",
            ],
            vec!["bijux-dev-atlas", "check", "doctor"],
            vec!["bijux-dev-atlas", "check", "run", "--suite", "ci_fast"],
        ];
        for argv in commands {
            let cli = crate::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(crate::cli::Command::Check { .. }) => {}
                _ => panic!("expected check command"),
            }
        }
    }

    #[test]
    fn release_subcommands_parse() {
        let cli =
            crate::Cli::try_parse_from(vec!["bijux-dev-atlas", "release", "check"]).expect("parse");
        match cli.command {
            Some(crate::cli::Command::Release { .. }) => {}
            _ => panic!("expected release command"),
        }
    }

    #[test]
    fn top_level_version_and_help_inventory_parse() {
        for argv in [
            vec!["bijux-dev-atlas", "version"],
            vec!["bijux-dev-atlas", "version", "--format", "json"],
            vec!["bijux-dev-atlas", "help"],
            vec!["bijux-dev-atlas", "help", "--format", "json"],
            vec!["bijux-dev-atlas", "--print-boundaries"],
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
            vec!["bijux-dev-atlas", "docs", "doctor"],
            vec!["bijux-dev-atlas", "docs", "check", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "docs", "verify-contracts"],
            vec!["bijux-dev-atlas", "docs", "validate"],
            vec!["bijux-dev-atlas", "docs", "clean", "--allow-write"],
            vec!["bijux-dev-atlas", "docs", "lint"],
            vec!["bijux-dev-atlas", "docs", "links"],
            vec!["bijux-dev-atlas", "docs", "inventory"],
            vec!["bijux-dev-atlas", "docs", "grep", "bijux dev atlas"],
            vec![
                "bijux-dev-atlas",
                "docs",
                "build",
                "--allow-subprocess",
                "--allow-write",
                "--strict",
            ],
            vec![
                "bijux-dev-atlas",
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
    #[ignore = "mkdocs nav legacy parser pending rewrite"]
    fn mkdocs_nav_parser_extracts_refs() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docs-mini");
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
                "start-here.md".to_string(),
            ]
        );
    }

    #[test]
    fn docs_link_resolver_accepts_fixture_links() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docs-mini");
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
            vec!["bijux-dev-atlas", "configs", "doctor"],
            vec!["bijux-dev-atlas", "configs", "print"],
            vec!["bijux-dev-atlas", "configs", "list"],
            vec!["bijux-dev-atlas", "configs", "verify"],
            vec!["bijux-dev-atlas", "configs", "validate", "--strict"],
            vec!["bijux-dev-atlas", "configs", "lint"],
            vec!["bijux-dev-atlas", "configs", "fmt", "--check"],
            vec!["bijux-dev-atlas", "configs", "inventory"],
            vec!["bijux-dev-atlas", "configs", "compile", "--allow-write"],
            vec!["bijux-dev-atlas", "configs", "diff"],
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
            vec!["bijux-dev-atlas", "docker", "build", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "docker", "check", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "docker", "smoke", "--allow-subprocess"],
            vec![
                "bijux-dev-atlas",
                "docker",
                "scan",
                "--allow-subprocess",
                "--allow-network",
            ],
            vec!["bijux-dev-atlas", "docker", "sbom", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "docker", "policy", "check"],
            vec!["bijux-dev-atlas", "docker", "lock", "--allow-write"],
            vec![
                "bijux-dev-atlas",
                "docker",
                "push",
                "--allow-subprocess",
                "--allow-network",
                "--i-know-what-im-doing",
            ],
            vec![
                "bijux-dev-atlas",
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
    fn build_subcommands_parse() {
        let commands = [
            vec!["bijux-dev-atlas", "build", "bin"],
            vec![
                "bijux-dev-atlas",
                "build",
                "dist",
                "--allow-subprocess",
                "--allow-write",
            ],
            vec!["bijux-dev-atlas", "build", "doctor", "--format", "json"],
            vec!["bijux-dev-atlas", "build", "clean", "--allow-write"],
            vec![
                "bijux-dev-atlas",
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
            vec!["bijux-dev-atlas", "policies", "list"],
            vec!["bijux-dev-atlas", "policies", "explain", "repo"],
            vec!["bijux-dev-atlas", "policies", "report"],
            vec!["bijux-dev-atlas", "policies", "print"],
            vec![
                "bijux-dev-atlas",
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
        let commands = [vec!["bijux-dev-atlas", "workflows", "validate"]];
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
            vec!["bijux-dev-atlas", "gates", "list"],
            vec!["bijux-dev-atlas", "gates", "run"],
            vec!["bijux-dev-atlas", "gates", "run", "--suite", "deep"],
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
