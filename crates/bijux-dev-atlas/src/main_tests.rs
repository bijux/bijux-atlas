#[cfg(test)]
mod tests {
    use clap::Parser;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn source_does_not_reference_atlasctl_runtime() {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let forbidden_python_module = ["python -m ", "atlasctl"].concat();
        let forbidden_wrapper = ["/bin/", "atlasctl"].concat();
        let mut stack = vec![src];
        while let Some(path) = stack.pop() {
            for entry in fs::read_dir(path).expect("read_dir") {
                let entry = entry.expect("entry");
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                    continue;
                }
                let text = fs::read_to_string(&path).expect("read file");
                assert!(
                    !text.contains(&forbidden_python_module),
                    "new rust dev tool must not invoke python atlas runtime: {}",
                    path.display()
                );
                assert!(
                    !text.contains(&forbidden_wrapper),
                    "new rust dev tool must not invoke atlasctl binary wrapper: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn ops_subcommands_parse() {
        let commands = [
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
            ],
            vec!["bijux-dev-atlas", "ops", "down", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "clean"],
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
    fn docs_subcommands_parse() {
        let commands = [
            vec!["bijux-dev-atlas", "docs", "doctor"],
            vec!["bijux-dev-atlas", "docs", "validate"],
            vec!["bijux-dev-atlas", "docs", "lint"],
            vec!["bijux-dev-atlas", "docs", "links"],
            vec!["bijux-dev-atlas", "docs", "inventory"],
            vec!["bijux-dev-atlas", "docs", "grep", "atlasctl"],
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docs-mini");
        let refs = crate::mkdocs_nav_refs(&root).expect("mkdocs nav");
        let paths = refs.into_iter().map(|(_, p)| p).collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec!["index.md".to_string(), "sub/intro.md".to_string()]
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
            vec!["bijux-dev-atlas", "configs", "validate", "--strict"],
            vec!["bijux-dev-atlas", "configs", "lint"],
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
    fn policies_subcommands_parse() {
        let commands = [
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
