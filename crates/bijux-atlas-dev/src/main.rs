#![forbid(unsafe_code)]

use std::path::PathBuf;

use bijux_atlas_dev_adapters::{Capabilities, RealProcessRunner};
use bijux_atlas_dev_core::{
    explain_output, list_output, load_registry, registry_doctor, run_checks, select_checks,
    RunRequest, Selectors,
};
use bijux_atlas_dev_model::{CheckId, DomainId};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "bijux-atlas-dev", version)]
#[command(about = "Bijux Atlas development control-plane")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Check {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum)]
        domain: Option<DomainArg>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        explain: Option<String>,
        #[arg(long)]
        suite: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        id_glob: Option<String>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        allow_fs_write: bool,
        #[arg(long, default_value_t = false)]
        allow_subprocess: bool,
        #[arg(long, default_value_t = false)]
        allow_git: bool,
        #[arg(long, default_value_t = false)]
        allow_network: bool,
    },
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DomainArg {
    Ops,
    Repo,
    Docs,
    Make,
}

impl From<DomainArg> for DomainId {
    fn from(value: DomainArg) -> Self {
        match value {
            DomainArg::Ops => Self::Ops,
            DomainArg::Repo => Self::Repo,
            DomainArg::Docs => Self::Docs,
            DomainArg::Make => Self::Make,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let exit = match cli.command {
        Command::Doctor { repo_root } => {
            let root = repo_root
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            let report = registry_doctor(&root);
            if report.errors.is_empty() {
                println!("bijux-atlas-dev doctor: ok");
                0
            } else {
                eprintln!("bijux-atlas-dev doctor failed:");
                for err in report.errors {
                    eprintln!("{err}");
                }
                1
            }
        }
        Command::Check {
            repo_root,
            domain,
            list,
            explain,
            suite,
            tag,
            id_glob,
            include_internal,
            allow_fs_write,
            allow_subprocess,
            allow_git,
            allow_network,
        } => {
            let root = repo_root
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            match load_registry(&root) {
                Ok(registry) => match (|| -> Result<Selectors, String> {
                    Ok(Selectors {
                        id_glob,
                        domain: domain.map(Into::into),
                        tag: tag
                            .map(|v| bijux_atlas_dev_model::Tag::parse(&v))
                            .transpose()?,
                        suite: suite
                            .map(|v| bijux_atlas_dev_model::SuiteId::parse(&v))
                            .transpose()?,
                        include_internal,
                    })
                })() {
                    Ok(selectors) => {
                        if list {
                            match select_checks(&registry, &selectors) {
                                Ok(checks) => {
                                    println!("{}", list_output(&checks));
                                    0
                                }
                                Err(err) => {
                                    eprintln!("bijux-atlas-dev check failed: {err}");
                                    1
                                }
                            }
                        } else if let Some(check_id) = explain {
                            match CheckId::parse(&check_id)
                                .and_then(|id| explain_output(&registry, &id))
                            {
                                Ok(text) => {
                                    println!("{text}");
                                    0
                                }
                                Err(err) => {
                                    eprintln!("bijux-atlas-dev check failed: {err}");
                                    1
                                }
                            }
                        } else {
                            let request = RunRequest {
                                repo_root: root,
                                domain: selectors.domain,
                                capabilities: Capabilities::from_cli_flags(
                                    allow_fs_write,
                                    allow_subprocess,
                                    allow_git,
                                    allow_network,
                                ),
                            };
                            let adapter = RealProcessRunner;
                            match run_checks(&adapter, &request) {
                                Ok(results) => {
                                    println!(
                                        "{}",
                                        serde_json::to_string_pretty(&results)
                                            .unwrap_or_else(|_| "{}".to_string())
                                    );
                                    0
                                }
                                Err(err) => {
                                    eprintln!("bijux-atlas-dev check failed: {err}");
                                    1
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("bijux-atlas-dev check failed: {err}");
                        1
                    }
                },
                Err(err) => {
                    eprintln!("bijux-atlas-dev check failed: {err}");
                    1
                }
            }
        }
    };
    std::process::exit(exit);
}

#[cfg(test)]
mod tests {
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
}
