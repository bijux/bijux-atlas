use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--locked")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run cargo metadata for workspace root");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid cargo metadata JSON");
    PathBuf::from(
        v.get("workspace_root")
            .and_then(serde_json::Value::as_str)
            .expect("workspace_root missing from cargo metadata"),
    )
}

fn parse_boundaries(boundaries_text: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut allowed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for raw_line in boundaries_text.lines() {
        let line = raw_line.trim();
        if !line.starts_with("- `bijux-atlas-") || !line.contains(" -> ") {
            continue;
        }
        let line = line.trim_start_matches("- ");
        let (from_part, to_part) = line
            .split_once(" -> ")
            .expect("boundary line must contain direction");
        let from = from_part.trim().trim_matches('`').to_string();

        let to_clean = to_part.trim_end_matches('.').trim();
        let mut deps = BTreeSet::new();
        for dep in to_clean.split(',') {
            let dep = dep.trim().trim_matches('`');
            if dep.starts_with("bijux-atlas-") {
                deps.insert(dep.to_string());
            }
        }
        allowed.insert(from, deps);
    }

    allowed
}

fn internal_edges_from_metadata(root: &Path) -> BTreeMap<String, BTreeSet<String>> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--locked")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .current_dir(root)
        .output()
        .expect("failed to run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata returned invalid JSON");
    let packages = v
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .expect("metadata packages missing");

    let mut internal = BTreeSet::new();
    for pkg in packages {
        let name = pkg
            .get("name")
            .and_then(serde_json::Value::as_str)
            .expect("package name missing");
        if name.starts_with("bijux-atlas-") {
            internal.insert(name.to_string());
        }
    }

    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for pkg in packages {
        let name = pkg
            .get("name")
            .and_then(serde_json::Value::as_str)
            .expect("package name missing")
            .to_string();
        if !internal.contains(&name) {
            continue;
        }

        let mut deps = BTreeSet::new();
        for dep in pkg
            .get("dependencies")
            .and_then(serde_json::Value::as_array)
            .expect("dependencies missing")
        {
            let dep_name = dep
                .get("name")
                .and_then(serde_json::Value::as_str)
                .expect("dependency name missing");
            if internal.contains(dep_name) {
                deps.insert(dep_name.to_string());
            }
        }
        edges.insert(name, deps);
    }

    edges
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };
    for entry in entries {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rs_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}

#[test]
fn crate_dependency_dag_matches_boundaries_doc() {
    let root = workspace_root();
    let boundaries = fs::read_to_string(root.join("docs/architecture/boundaries.md"))
        .expect("missing docs/architecture/boundaries.md");
    let allowed = parse_boundaries(&boundaries);
    let actual = internal_edges_from_metadata(&root);

    for (crate_name, deps) in &actual {
        if let Some(allowed_deps) = allowed.get(crate_name) {
            for dep in deps {
                assert!(
                    allowed_deps.contains(dep),
                    "disallowed internal dependency: {crate_name} -> {dep}"
                );
            }
        } else {
            assert!(
                deps.is_empty(),
                "crate missing from boundaries.md: {crate_name}"
            );
        }
    }
}

#[test]
fn api_and_server_must_not_spawn_processes() {
    let root = workspace_root();
    let targets = [
        root.join("crates/bijux-atlas-api/src"),
        root.join("crates/bijux-atlas-server/src"),
    ];

    let forbidden = ["Command::new", "tokio::process", "duct::"];
    for target in targets {
        for file in collect_rs_files(&target) {
            let content = fs::read_to_string(&file).expect("failed to read rust file");
            for needle in forbidden {
                assert!(
                    !content.contains(needle),
                    "forbidden process spawn token `{needle}` in {}",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn override_and_escape_hatches_are_forbidden() {
    let root = workspace_root();
    let targets = [
        root.join("crates/bijux-atlas-core/src"),
        root.join("crates/bijux-atlas-model/src"),
        root.join("crates/bijux-atlas-ingest/src"),
        root.join("crates/bijux-atlas-store/src"),
        root.join("crates/bijux-atlas-query/src"),
        root.join("crates/bijux-atlas-api/src"),
        root.join("crates/bijux-atlas-cli/src"),
        root.join("crates/bijux-atlas-server/src"),
    ];
    let forbidden = [
        "escape_hatch",
        "allow_override",
        "force_override",
        "unsafe_bypass",
    ];

    for target in targets {
        for file in collect_rs_files(&target) {
            let content = fs::read_to_string(&file).expect("failed to read rust file");
            for needle in forbidden {
                assert!(
                    !content.contains(needle),
                    "forbidden override/escape token `{needle}` in {}",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn server_must_not_depend_on_ingest_crate() {
    let root = workspace_root();
    let cargo_toml = fs::read_to_string(root.join("crates/bijux-atlas-server/Cargo.toml"))
        .expect("missing crates/bijux-atlas-server/Cargo.toml");
    assert!(
        !cargo_toml.contains("bijux-atlas-ingest"),
        "server crate must not depend on ingest internals"
    );
}

#[test]
fn query_layer_must_not_depend_on_runtime_network_or_async_stacks() {
    let root = workspace_root();
    let cargo_toml = fs::read_to_string(root.join("crates/bijux-atlas-query/Cargo.toml"))
        .expect("missing crates/bijux-atlas-query/Cargo.toml");
    for forbidden in ["tokio", "reqwest", "axum", "hyper"] {
        assert!(
            !cargo_toml.contains(forbidden),
            "query layer must stay pure and cannot depend on runtime/net stack: {forbidden}"
        );
    }
}

#[test]
fn api_layer_cannot_read_raw_gff3_or_fasta() {
    let root = workspace_root();
    let api_src = root.join("crates/bijux-atlas-api/src");
    let forbidden = [
        ".gff3",
        ".gff",
        ".fa",
        ".fasta",
        "File::open",
        "fs::read",
        "fs::read_to_string",
    ];

    for file in collect_rs_files(&api_src) {
        let content = fs::read_to_string(&file).expect("failed to read rust file");
        for needle in forbidden {
            assert!(
                !content.contains(needle),
                "forbidden raw input token `{needle}` in {}",
                file.display()
            );
        }
    }
}

#[test]
fn unit_tests_must_not_use_network_calls() {
    let root = workspace_root();
    let crates = root.join("crates");
    let forbidden = [
        "reqwest",
        "ureq",
        "TcpStream::connect",
        "UdpSocket::bind",
        "hyper::",
        "surf::",
        "isahc::",
    ];

    for crate_dir in fs::read_dir(crates).expect("failed to read crates dir") {
        let crate_dir = crate_dir.expect("dir entry failed").path();
        let src_dir = crate_dir.join("src");
        for file in collect_rs_files(&src_dir) {
            let content = fs::read_to_string(&file).expect("failed to read rust file");
            if !content.contains("#[cfg(test)]") {
                continue;
            }
            let test_section = content
                .split("#[cfg(test)]")
                .nth(1)
                .unwrap_or_default()
                .to_string();
            for needle in forbidden {
                assert!(
                    !test_section.contains(needle),
                    "network token `{needle}` is forbidden in unit tests: {}",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn atlas_repo_must_not_define_umbrella_bijux_binary() {
    let root = workspace_root();
    let cargo_tomls = [
        root.join("Cargo.toml"),
        root.join("crates/bijux-atlas-cli/Cargo.toml"),
        root.join("crates/bijux-atlas-server/Cargo.toml"),
        root.join("crates/bijux-atlas-api/Cargo.toml"),
    ];

    for file in cargo_tomls {
        let content = fs::read_to_string(&file).expect("failed to read Cargo.toml");
        assert!(
            !content.contains("name = \"bijux\""),
            "atlas repo must not define umbrella binary `bijux` in {}",
            file.display()
        );
    }
}

#[test]
fn atlas_must_not_depend_on_bijux_dna_crates() {
    let root = workspace_root();
    let cargo_tomls = collect_files_by_name(&root, "Cargo.toml");
    for file in cargo_tomls {
        let content = fs::read_to_string(&file).expect("failed to read Cargo.toml");
        assert!(
            !content.contains("bijux-dna"),
            "atlas must not reference bijux-dna in {}",
            file.display()
        );
    }
}

fn collect_files_by_name(dir: &Path, name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|v| v == "target") {
                continue;
            }
            out.extend(collect_files_by_name(&path, name));
        } else if path.file_name().is_some_and(|v| v == name) {
            out.push(path);
        }
    }
    out
}

#[test]
fn ingestion_must_be_pure_transform_only() {
    let root = workspace_root();
    let ingest_src = root.join("crates/bijux-atlas-ingest/src");
    let forbidden = [
        "reqwest",
        "ureq",
        "hyper::",
        "postgres",
        "mysql",
        "redis",
        "std::net",
        "Command::new",
        "tokio::process",
    ];

    for file in collect_rs_files(&ingest_src) {
        let content = fs::read_to_string(&file).expect("failed to read rust file");
        for needle in forbidden {
            assert!(
                !content.contains(needle),
                "forbidden side-effect token `{needle}` in ingest crate: {}",
                file.display()
            );
        }
    }
}

#[test]
fn server_http_layers_must_not_read_raw_files_directly() {
    let root = workspace_root();
    let http_src = root.join("crates/bijux-atlas-server/src/http");
    let forbidden = [
        "std::fs::",
        "fs::read(",
        "fs::read_to_string(",
        "File::open(",
    ];
    for file in collect_rs_files(&http_src) {
        if file
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "effects_adapters.rs")
        {
            continue;
        }
        let content = fs::read_to_string(&file).expect("failed to read rust file");
        for needle in forbidden {
            assert!(
                !content.contains(needle),
                "http layer must use cache/store adapters for file IO; found `{needle}` in {}",
                file.display()
            );
        }
    }
}
