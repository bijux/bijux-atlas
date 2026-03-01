// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

const RUNTIME_CRATES: [&str; 9] = [
    "bijux-atlas-api",
    "bijux-atlas-cli",
    "bijux-atlas-core",
    "bijux-atlas-ingest",
    "bijux-atlas-model",
    "bijux-atlas-policies",
    "bijux-atlas-query",
    "bijux-atlas-server",
    "bijux-atlas-store",
];

fn runtime_manifest_paths(repo_root: &Path) -> Vec<PathBuf> {
    RUNTIME_CRATES
        .iter()
        .map(|name| repo_root.join("crates").join(name).join("Cargo.toml"))
        .collect()
}

fn allowed_runtime_deps() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    BTreeMap::from([
        (
            "bijux-atlas-api",
            BTreeSet::from(["bijux-atlas-core", "bijux-atlas-model", "bijux-atlas-query"]),
        ),
        (
            "bijux-atlas-cli",
            BTreeSet::from([
                "bijux-atlas-core",
                "bijux-atlas-ingest",
                "bijux-atlas-model",
                "bijux-atlas-policies",
                "bijux-atlas-query",
                "bijux-atlas-store",
            ]),
        ),
        ("bijux-atlas-core", BTreeSet::new()),
        (
            "bijux-atlas-ingest",
            BTreeSet::from(["bijux-atlas-core", "bijux-atlas-model"]),
        ),
        ("bijux-atlas-model", BTreeSet::new()),
        ("bijux-atlas-policies", BTreeSet::new()),
        (
            "bijux-atlas-query",
            BTreeSet::from([
                "bijux-atlas-core",
                "bijux-atlas-model",
                "bijux-atlas-policies",
                "bijux-atlas-store",
            ]),
        ),
        (
            "bijux-atlas-server",
            BTreeSet::from([
                "bijux-atlas-api",
                "bijux-atlas-core",
                "bijux-atlas-model",
                "bijux-atlas-query",
                "bijux-atlas-store",
            ]),
        ),
        (
            "bijux-atlas-store",
            BTreeSet::from(["bijux-atlas-core", "bijux-atlas-model"]),
        ),
    ])
}

fn parse_runtime_deps(manifest_path: &Path) -> Result<(String, BTreeSet<String>), String> {
    let text = fs::read_to_string(manifest_path)
        .map_err(|err| format!("read {} failed: {err}", manifest_path.display()))?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|err| format!("parse {} failed: {err}", manifest_path.display()))?;
    let package = value
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("{} missing package.name", manifest_path.display()))?
        .to_string();
    let deps = value
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(|table| {
            table
                .keys()
                .filter(|dep| dep.starts_with("bijux-atlas-") || dep == &"bijux-dev-atlas")
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    Ok((package, deps))
}

fn push_violation(
    contract_id: &str,
    test_id: &str,
    file: Option<String>,
    message: impl Into<String>,
) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: None,
        message: message.into(),
        evidence: None,
    }
}

fn collect_rs_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries =
            fs::read_dir(dir).map_err(|err| format!("read {} failed: {err}", dir.display()))?;
        for entry in entries {
            let entry = entry
                .map_err(|err| format!("read_dir entry in {} failed: {err}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    if root.exists() {
        walk(root, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn file_contains_any(path: &Path, needles: &[&str]) -> Result<Vec<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    Ok(needles
        .iter()
        .filter(|needle| text.contains(**needle))
        .map(|needle| (*needle).to_string())
        .collect())
}

fn relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn api_surface_lines(repo_root: &Path) -> Result<Vec<String>, String> {
    let lib = repo_root.join("crates/bijux-atlas-api/src/lib.rs");
    let text =
        fs::read_to_string(&lib).map_err(|err| format!("read {} failed: {err}", lib.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("pub const ")
                || line.starts_with("pub mod ")
                || line.starts_with("pub use ")
                || line.starts_with("pub fn ")
        })
        .map(ToOwned::to_owned)
        .collect())
}

fn test_runtime_001_dependency_allowlist(ctx: &RunContext) -> TestResult {
    let allowed = allowed_runtime_deps();
    let mut violations = Vec::new();
    for manifest in runtime_manifest_paths(&ctx.repo_root) {
        let (package, deps) = match parse_runtime_deps(&manifest) {
            Ok(value) => value,
            Err(err) => {
                violations.push(push_violation(
                    "RUNTIME-001",
                    "runtime.deps.allowlist",
                    Some(relative_to_repo(&ctx.repo_root, &manifest)),
                    err,
                ));
                continue;
            }
        };
        let expected = allowed
            .get(package.as_str())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        for dep in deps.iter().filter(|dep| dep.starts_with("bijux-atlas-")) {
            if !expected.contains(dep) {
                violations.push(push_violation(
                    "RUNTIME-001",
                    "runtime.deps.allowlist",
                    Some(relative_to_repo(&ctx.repo_root, &manifest)),
                    format!("runtime crate `{package}` depends on undeclared crate `{dep}`"),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_runtime_002_server_no_ingest_dep(ctx: &RunContext) -> TestResult {
    let manifest = ctx.repo_root.join("crates/bijux-atlas-server/Cargo.toml");
    match parse_runtime_deps(&manifest) {
        Ok((package, deps)) => {
            if deps.contains("bijux-atlas-ingest") {
                TestResult::Fail(vec![push_violation(
                    "RUNTIME-002",
                    "runtime.server.no_ingest_dep",
                    Some(relative_to_repo(&ctx.repo_root, &manifest)),
                    format!("runtime crate `{package}` must not depend on bijux-atlas-ingest"),
                )])
            } else {
                TestResult::Pass
            }
        }
        Err(err) => TestResult::Fail(vec![push_violation(
            "RUNTIME-002",
            "runtime.server.no_ingest_dep",
            Some(relative_to_repo(&ctx.repo_root, &manifest)),
            err,
        )]),
    }
}

fn test_runtime_003_no_control_plane_dep(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for manifest in runtime_manifest_paths(&ctx.repo_root) {
        match parse_runtime_deps(&manifest) {
            Ok((package, deps)) => {
                if deps.contains("bijux-dev-atlas") {
                    violations.push(push_violation(
                        "RUNTIME-003",
                        "runtime.no_control_plane_dep",
                        Some(relative_to_repo(&ctx.repo_root, &manifest)),
                        format!("runtime crate `{package}` must not depend on bijux-dev-atlas"),
                    ));
                }
            }
            Err(err) => violations.push(push_violation(
                "RUNTIME-003",
                "runtime.no_control_plane_dep",
                Some(relative_to_repo(&ctx.repo_root, &manifest)),
                err,
            )),
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_runtime_004_pure_crates_no_host_io(ctx: &RunContext) -> TestResult {
    let crates = [
        "bijux-atlas-core",
        "bijux-atlas-model",
        "bijux-atlas-policies",
        "bijux-atlas-api",
    ];
    let forbidden = [
        "std::fs",
        "tokio::fs",
        "reqwest",
        "std::process::Command",
        "ProcessCommand::new",
        "TcpStream::connect",
        "UdpSocket::bind",
        "std::net::",
        "tokio::net::",
        "hyper::",
        "axum::",
    ];
    let mut violations = Vec::new();
    for crate_name in crates {
        let src_root = ctx.repo_root.join("crates").join(crate_name).join("src");
        let files = match collect_rs_files(&src_root) {
            Ok(value) => value,
            Err(err) => {
                violations.push(push_violation(
                    "RUNTIME-004",
                    "runtime.pure_crates.no_host_io",
                    Some(relative_to_repo(&ctx.repo_root, &src_root)),
                    err,
                ));
                continue;
            }
        };
        for file in files {
            let relative = relative_to_repo(&ctx.repo_root, &file);
            if relative.contains("/src/bin/") || relative.ends_with("/adapters.rs") {
                continue;
            }
            let hits = match file_contains_any(&file, &forbidden) {
                Ok(value) => value,
                Err(err) => {
                    violations.push(push_violation(
                        "RUNTIME-004",
                        "runtime.pure_crates.no_host_io",
                        Some(relative.clone()),
                        err,
                    ));
                    continue;
                }
            };
            for hit in hits {
                violations.push(push_violation(
                    "RUNTIME-004",
                    "runtime.pure_crates.no_host_io",
                    Some(relative.clone()),
                    format!(
                        "pure runtime crate `{crate_name}` must not use host I/O marker `{hit}`"
                    ),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_runtime_005_api_surface_snapshot(ctx: &RunContext) -> TestResult {
    let snapshot = ctx
        .repo_root
        .join("crates/bijux-dev-atlas/tests/goldens/runtime_api_surface.txt");
    let actual = match api_surface_lines(&ctx.repo_root) {
        Ok(lines) => format!("{}\n", lines.join("\n")),
        Err(err) => {
            return TestResult::Fail(vec![push_violation(
                "RUNTIME-005",
                "runtime.api.surface_snapshot",
                Some("crates/bijux-atlas-api/src/lib.rs".to_string()),
                err,
            )]);
        }
    };
    let expected = match fs::read_to_string(&snapshot) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![push_violation(
                "RUNTIME-005",
                "runtime.api.surface_snapshot",
                Some(relative_to_repo(&ctx.repo_root, &snapshot)),
                format!("read snapshot failed: {err}"),
            )]);
        }
    };
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![push_violation(
            "RUNTIME-005",
            "runtime.api.surface_snapshot",
            Some(relative_to_repo(&ctx.repo_root, &snapshot)),
            "runtime API surface drifted from the committed snapshot",
        )])
    }
}

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    if !repo_root
        .join("crates/bijux-atlas-core/Cargo.toml")
        .exists()
    {
        return Err("runtime contracts require the workspace crates surface".to_string());
    }
    Ok(vec![
        Contract {
            id: ContractId("RUNTIME-001".to_string()),
            title: "runtime dependency layering stays on the approved graph",
            tests: vec![TestCase {
                id: TestId("runtime.deps.allowlist".to_string()),
                title: "runtime crate dependencies stay on the declared allowlist graph",
                kind: TestKind::Pure,
                run: test_runtime_001_dependency_allowlist,
            }],
        },
        Contract {
            id: ContractId("RUNTIME-002".to_string()),
            title: "server stays out of the ingest layer",
            tests: vec![TestCase {
                id: TestId("runtime.server.no_ingest_dependency".to_string()),
                title: "bijux-atlas-server does not depend on bijux-atlas-ingest",
                kind: TestKind::Pure,
                run: test_runtime_002_server_no_ingest_dep,
            }],
        },
        Contract {
            id: ContractId("RUNTIME-003".to_string()),
            title: "runtime crates do not depend on the control plane",
            tests: vec![TestCase {
                id: TestId("runtime.deps.no_control_plane_dependency".to_string()),
                title: "runtime manifests do not import bijux-dev-atlas",
                kind: TestKind::Pure,
                run: test_runtime_003_no_control_plane_dep,
            }],
        },
        Contract {
            id: ContractId("RUNTIME-004".to_string()),
            title: "pure runtime crates avoid host I/O",
            tests: vec![TestCase {
                id: TestId("runtime.pure_crates.no_host_io".to_string()),
                title: "core model policies and api sources avoid direct host I/O markers",
                kind: TestKind::Pure,
                run: test_runtime_004_pure_crates_no_host_io,
            }],
        },
        Contract {
            id: ContractId("RUNTIME-005".to_string()),
            title: "runtime api surface stays stable",
            tests: vec![TestCase {
                id: TestId("runtime.api.surface_snapshot".to_string()),
                title: "bijux-atlas-api public surface matches the committed snapshot",
                kind: TestKind::Pure,
                run: test_runtime_005_api_surface_snapshot,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "RUNTIME-001" => {
            "Runtime crate dependencies must stay on the approved layering graph.".to_string()
        }
        "RUNTIME-002" => {
            "The server runtime must not depend directly on the ingest layer.".to_string()
        }
        "RUNTIME-003" => {
            "Runtime crates may not import the bijux-dev-atlas control-plane crate.".to_string()
        }
        "RUNTIME-004" => {
            "Pure runtime crates must avoid direct filesystem, network, and subprocess usage."
                .to_string()
        }
        "RUNTIME-005" => {
            "The public bijux-atlas-api surface is snapshot-governed and must move intentionally."
                .to_string()
        }
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts runtime`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts runtime --mode static"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_surface_snapshot_source_is_not_empty() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace = match manifest_dir.parent() {
            Some(path) => path,
            None => panic!("workspace parent missing"),
        };
        let repo_root = match workspace.parent() {
            Some(path) => path.to_path_buf(),
            None => panic!("repo root missing"),
        };
        let lines = match api_surface_lines(&repo_root) {
            Ok(lines) => lines,
            Err(err) => panic!("surface: {err}"),
        };
        assert!(!lines.is_empty());
    }
}
