#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use bijux_dev_atlas_adapters::{Capabilities, RealFs, RealProcessRunner};
use bijux_dev_atlas_core::{
    exit_code_for_report, explain_output, list_output, load_registry, registry_doctor, render_json,
    render_jsonl, render_text_with_durations, run_checks, select_checks, RunOptions, RunRequest,
    Selectors,
};
use bijux_dev_atlas_model::{CheckId, DomainId, RunId, SuiteId, Tag};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "bijux-dev-atlas", version)]
#[command(about = "Bijux Atlas development control-plane")]
struct Cli {
    #[arg(long, default_value_t = false)]
    quiet: bool,
    #[arg(long, default_value_t = false)]
    verbose: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        suite: Option<String>,
        #[arg(long, value_enum)]
        domain: Option<DomainArg>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_name = "GLOB")]
        id: Option<String>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Explain {
        check_id: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_root: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        suite: Option<String>,
        #[arg(long, value_enum)]
        domain: Option<DomainArg>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_name = "GLOB")]
        id: Option<String>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, default_value_t = false)]
        allow_subprocess: bool,
        #[arg(long, default_value_t = false)]
        allow_git: bool,
        #[arg(long = "allow-write", default_value_t = false)]
        allow_write: bool,
        #[arg(long, default_value_t = false)]
        allow_network: bool,
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long)]
        max_failures: Option<usize>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = 0)]
        durations: usize,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DomainArg {
    Ops,
    Repo,
    Docs,
    Make,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FormatArg {
    Text,
    Json,
    Jsonl,
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

fn discover_repo_root(start: &Path) -> Result<PathBuf, String> {
    let mut current = start.canonicalize().map_err(|err| err.to_string())?;
    loop {
        if current.join("ops/atlas-dev/registry.toml").exists() {
            return Ok(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Err(
                "could not discover repo root (no ops/atlas-dev/registry.toml found)"
                    .to_string(),
            );
        }
    }
}

fn resolve_repo_root(arg: Option<PathBuf>) -> Result<PathBuf, String> {
    match arg {
        Some(path) => discover_repo_root(&path),
        None => {
            let cwd = std::env::current_dir().map_err(|err| err.to_string())?;
            discover_repo_root(&cwd)
        }
    }
}

fn parse_selectors(
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
) -> Result<Selectors, String> {
    Ok(Selectors {
        suite: suite.map(|v| SuiteId::parse(&v)).transpose()?,
        domain: domain.map(Into::into),
        tag: tag.map(|v| Tag::parse(&v)).transpose()?,
        id_glob: id,
        include_internal,
        include_slow,
    })
}

fn write_output_if_requested(out: Option<PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = out {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    }
    Ok(())
}

fn render_list_output(checks_text: String, format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => Ok(checks_text),
        FormatArg::Json => {
            let rows: Vec<serde_json::Value> = checks_text
                .lines()
                .filter_map(|line| {
                    let (id, title) = line.split_once('\t')?;
                    Some(serde_json::json!({"id": id, "title": title}))
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({"checks": rows}))
                .map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for list".to_string()),
    }
}

fn render_explain_output(explain_text: String, format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => Ok(explain_text),
        FormatArg::Json => {
            let mut map = serde_json::Map::new();
            for line in explain_text.lines() {
                if let Some((key, value)) = line.split_once(": ") {
                    map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
            serde_json::to_string_pretty(&serde_json::Value::Object(map)).map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for explain".to_string()),
    }
}

fn render_doctor_output(
    report: &bijux_dev_atlas_core::RegistryDoctorReport,
    format: FormatArg,
) -> Result<String, String> {
    match format {
        FormatArg::Text => {
            if report.errors.is_empty() {
                Ok(String::new())
            } else {
                Ok(report.errors.join("\n"))
            }
        }
        FormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
            "status": if report.errors.is_empty() { "ok" } else { "failed" },
            "errors": report.errors,
        }))
        .map_err(|err| err.to_string()),
        FormatArg::Jsonl => Err("jsonl output is not supported for doctor".to_string()),
    }
}

fn main() {
    let cli = Cli::parse();
    let exit = match cli.command {
        Command::List {
            repo_root,
            suite,
            domain,
            tag,
            id,
            include_internal,
            include_slow,
            format,
            out,
        } => {
            match resolve_repo_root(repo_root).and_then(|root| {
                let selectors =
                    parse_selectors(suite, domain, tag, id, include_internal, include_slow)?;
                let registry = load_registry(&root)?;
                let checks = select_checks(&registry, &selectors)?;
                let rendered = render_list_output(list_output(&checks), format)?;
                write_output_if_requested(out, &rendered)?;
                Ok(rendered)
            }) {
                Ok(text) => {
                    if !cli.quiet && !text.is_empty() {
                        println!("{text}");
                    }
                    0
                }
                Err(err) => {
                    eprintln!("bijux-dev-atlas list failed: {err}");
                    1
                }
            }
        }
        Command::Explain {
            check_id,
            repo_root,
            format,
            out,
        } => match resolve_repo_root(repo_root).and_then(|root| {
            let registry = load_registry(&root)?;
            let id = CheckId::parse(&check_id)?;
            let rendered = render_explain_output(explain_output(&registry, &id)?, format)?;
            write_output_if_requested(out, &rendered)?;
            Ok(rendered)
        }) {
            Ok(text) => {
                if !cli.quiet && !text.is_empty() {
                    println!("{text}");
                }
                0
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas explain failed: {err}");
                1
            }
        },
        Command::Doctor {
            repo_root,
            format,
            out,
        } => match resolve_repo_root(repo_root) {
            Ok(root) => {
                let report = registry_doctor(&root);
                match render_doctor_output(&report, format).and_then(|rendered| {
                    write_output_if_requested(out, &rendered)?;
                    Ok(rendered)
                }) {
                    Ok(rendered) => {
                        if !cli.quiet && !rendered.is_empty() {
                            if report.errors.is_empty() {
                                println!("{rendered}");
                            } else {
                                eprintln!("{rendered}");
                            }
                        }
                        if report.errors.is_empty() { 0 } else { 1 }
                    }
                    Err(err) => {
                        eprintln!("bijux-dev-atlas doctor failed: {err}");
                        1
                    }
                }
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas doctor failed: {err}");
                1
            }
        },
        Command::Run {
            repo_root,
            artifacts_root,
            run_id,
            suite,
            domain,
            tag,
            id,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        } => {
            let result = resolve_repo_root(repo_root).and_then(|root| {
                let selectors =
                    parse_selectors(suite, domain, tag, id, include_internal, include_slow)?;
                let request = RunRequest {
                    repo_root: root.clone(),
                    domain: selectors.domain,
                    capabilities: Capabilities::from_cli_flags(
                        allow_write,
                        allow_subprocess,
                        allow_git,
                        allow_network,
                    ),
                    artifacts_root: artifacts_root.or_else(|| Some(root.join("artifacts"))),
                    run_id: run_id.map(|rid| RunId::parse(&rid)).transpose()?,
                };
                let options = RunOptions {
                    fail_fast,
                    max_failures,
                };
                let report =
                    run_checks(&RealProcessRunner, &RealFs, &request, &selectors, &options)?;
                let rendered = match format {
                    FormatArg::Text => render_text_with_durations(&report, durations),
                    FormatArg::Json => render_json(&report)?,
                    FormatArg::Jsonl => render_jsonl(&report)?,
                };
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, exit_code_for_report(&report)))
            });

            match result {
                Ok((rendered, code)) => {
                    if !cli.quiet {
                        println!("{rendered}");
                    }
                    code
                }
                Err(err) => {
                    eprintln!("bijux-dev-atlas run failed: {err}");
                    1
                }
            }
        }
    };

    if cli.verbose {
        eprintln!("bijux-dev-atlas exit={exit}");
    }
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
