#![forbid(unsafe_code)]

use std::path::PathBuf;

use bijux_atlas_dev_adapters::StdProcessAdapter;
use bijux_atlas_dev_core::{run_checks, RunRequest};
use bijux_atlas_dev_model::CheckDomain;
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
    },
    Doctor,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DomainArg {
    Ops,
    Repo,
    Docs,
    Make,
}

impl From<DomainArg> for CheckDomain {
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
        Command::Doctor => {
            println!("bijux-atlas-dev doctor: ok");
            0
        }
        Command::Check { repo_root, domain } => {
            let root = repo_root
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            let request = RunRequest {
                repo_root: root,
                domain: domain.map(Into::into),
            };
            let adapter = StdProcessAdapter;
            match run_checks(&adapter, &request) {
                Ok(results) => {
                    println!("{}", serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string()));
                    0
                }
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
