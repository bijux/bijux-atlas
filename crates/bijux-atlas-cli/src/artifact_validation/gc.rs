// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(crate) fn gc_plan(
    store_root: PathBuf,
    catalogs: Vec<PathBuf>,
    pins_path: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    refuse_gc_in_server_container()?;
    let report = compute_gc_plan(&store_root, &catalogs, &pins_path)?;
    emit_ok_payload(
        output_mode,
        serde_json::to_value(&report).map_err(|e| e.to_string())?,
    )
}

pub(crate) fn gc_apply(
    store_root: PathBuf,
    catalogs: Vec<PathBuf>,
    pins_path: PathBuf,
    confirm: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    refuse_gc_in_server_container()?;
    if !confirm {
        return Err("gc apply requires explicit --confirm; dry-run is default".to_string());
    }
    let mut report = compute_gc_plan(&store_root, &catalogs, &pins_path)?;
    let root = canonical_store_root(&store_root)?;
    for dataset_root in report.candidates.dataset_roots.clone() {
        let p = PathBuf::from(&dataset_root);
        ensure_within_root(&root, &p)?;
        match fs::remove_dir_all(&p) {
            Ok(()) => report.applied.deleted_dataset_roots.push(dataset_root),
            Err(e) => report
                .applied
                .errors
                .push(format!("delete {} failed: {e}", p.display())),
        }
    }
    report.applied.deleted_bytes = report
        .applied
        .deleted_dataset_roots
        .iter()
        .filter_map(|p| report.candidates.bytes_by_root.get(p).copied())
        .sum::<u64>();
    report.metrics = json!({
        "gc_candidates": report.candidates.dataset_roots.len(),
        "gc_deleted_bytes": report.applied.deleted_bytes,
        "gc_errors": report.applied.errors.len()
    });
    let out_dir = root.join("gc_reports");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let report_path = out_dir.join(format!("gc-report-{stamp}.json"));
    let bytes = canonical::stable_json_bytes(&report).map_err(|e| e.to_string())?;
    fs::write(&report_path, bytes).map_err(|e| e.to_string())?;
    report.report_path = Some(report_path.display().to_string());
    emit_ok_payload(
        output_mode,
        serde_json::to_value(&report).map_err(|e| e.to_string())?,
    )
}

fn refuse_gc_in_server_container() -> Result<(), String> {
    if std::env::var("ATLAS_SERVER_CONTAINER").ok().as_deref() == Some("1")
        || std::env::var("ATLAS_RUNTIME_ROLE")
            .ok()
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case("server"))
            .unwrap_or(false)
    {
        return Err("gc is CLI-only and is forbidden in server container/runtime".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct GcReport {
    pub(super) command: String,
    pub(super) status: String,
    pub(super) store_root: String,
    pub(super) catalogs: Vec<String>,
    pub(super) pins_path: String,
    pub(super) reachable: ReachableSummary,
    pub(super) candidates: CandidateSummary,
    pub(super) applied: AppliedSummary,
    pub(super) metrics: serde_json::Value,
    pub(super) report_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct ReachableSummary {
    pub(super) dataset_count: usize,
    pub(super) pinned_dataset_count: usize,
    pub(super) pinned_hash_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct CandidateSummary {
    pub(super) dataset_roots: Vec<String>,
    pub(super) bytes_by_root: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub(super) struct AppliedSummary {
    pub(super) deleted_dataset_roots: Vec<String>,
    pub(super) deleted_bytes: u64,
    pub(super) errors: Vec<String>,
}

pub(super) fn compute_gc_plan(
    store_root: &Path,
    catalogs: &[PathBuf],
    pins_path: &Path,
) -> Result<GcReport, String> {
    let root = canonical_store_root(store_root)?;
    let catalog_paths = resolve_catalog_paths(&root, catalogs)?;
    let pins = read_gc_pins(pins_path)?;

    let mut reachable_datasets: HashSet<DatasetId> = HashSet::new();
    for path in &catalog_paths {
        let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        catalog.validate_sorted().map_err(|e| e.to_string())?;
        for entry in catalog.datasets {
            reachable_datasets.insert(entry.dataset);
        }
    }
    for pinned in &pins.dataset_ids {
        if let Ok(d) = parse_dataset_key(pinned) {
            reachable_datasets.insert(d);
        }
    }

    let mut candidates = Vec::new();
    let mut bytes_by_root = BTreeMap::new();
    for dataset_root in discover_dataset_roots(&root)? {
        ensure_within_root(&root, &dataset_root)?;
        let Some(dataset) = dataset_id_from_root(&root, &dataset_root)? else {
            continue;
        };
        if reachable_datasets.contains(&dataset) {
            continue;
        }
        if let Some(manifest_hash) = read_manifest_hash_for_pin(&dataset_root)? {
            if pins.artifact_hashes.contains(&manifest_hash) {
                continue;
            }
        }
        let key = dataset_root.display().to_string();
        bytes_by_root.insert(key.clone(), dir_size_bytes(&dataset_root));
        candidates.push(key);
    }
    candidates.sort();
    Ok(GcReport {
        command: "atlas gc".to_string(),
        status: "ok".to_string(),
        store_root: root.display().to_string(),
        catalogs: catalog_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        pins_path: pins_path.display().to_string(),
        reachable: ReachableSummary {
            dataset_count: reachable_datasets.len(),
            pinned_dataset_count: pins.dataset_ids.len(),
            pinned_hash_count: pins.artifact_hashes.len(),
        },
        candidates: CandidateSummary {
            dataset_roots: candidates,
            bytes_by_root,
        },
        applied: AppliedSummary::default(),
        metrics: json!({}),
        report_path: None,
    })
}

fn canonical_store_root(store_root: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(store_root).map_err(|e| e.to_string())?;
    store_root
        .canonicalize()
        .map_err(|e| format!("store root canonicalize failed: {e}"))
}

fn resolve_catalog_paths(root: &Path, catalogs: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    if catalogs.is_empty() {
        return Ok(vec![root.join("catalog.json")]);
    }
    let mut out = Vec::new();
    for c in catalogs {
        let p = if c.is_absolute() {
            c.clone()
        } else {
            root.join(c)
        };
        out.push(p);
    }
    out.sort();
    out.dedup();
    Ok(out)
}

#[derive(Debug, Default)]
struct GcPins {
    dataset_ids: HashSet<String>,
    artifact_hashes: HashSet<String>,
}

fn read_gc_pins(path: &Path) -> Result<GcPins, String> {
    if !path.exists() {
        return Ok(GcPins::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let dataset_ids = v
        .get("dataset_ids")
        .and_then(serde_json::Value::as_array)
        .map(|xs| {
            xs.iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let artifact_hashes = v
        .get("artifact_hashes")
        .and_then(serde_json::Value::as_array)
        .map(|xs| {
            xs.iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    Ok(GcPins {
        dataset_ids,
        artifact_hashes,
    })
}

fn discover_dataset_roots(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let releases = fs::read_dir(root).map_err(|e| e.to_string())?;
    for rel in releases {
        let rel = rel.map_err(|e| e.to_string())?.path();
        if !rel.is_dir() {
            continue;
        }
        if !rel
            .file_name()
            .and_then(|x| x.to_str())
            .map(|x| x.starts_with("release="))
            .unwrap_or(false)
        {
            continue;
        }
        for species in fs::read_dir(&rel).map_err(|e| e.to_string())? {
            let species = species.map_err(|e| e.to_string())?.path();
            if !species.is_dir() {
                continue;
            }
            for assembly in fs::read_dir(&species).map_err(|e| e.to_string())? {
                let assembly = assembly.map_err(|e| e.to_string())?.path();
                if assembly.is_dir() {
                    out.push(assembly);
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

fn dataset_id_from_root(root: &Path, dataset_root: &Path) -> Result<Option<DatasetId>, String> {
    let rel = dataset_root
        .strip_prefix(root)
        .map_err(|e| format!("strip_prefix failed: {e}"))?;
    let parts = rel.iter().filter_map(|x| x.to_str()).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Ok(None);
    }
    let release = parts[0].strip_prefix("release=").unwrap_or("");
    let species = parts[1].strip_prefix("species=").unwrap_or("");
    let assembly = parts[2].strip_prefix("assembly=").unwrap_or("");
    if release.is_empty() || species.is_empty() || assembly.is_empty() {
        return Ok(None);
    }
    DatasetId::new(release, species, assembly)
        .map(Some)
        .map_err(|e| e.to_string())
}

fn read_manifest_hash_for_pin(dataset_root: &Path) -> Result<Option<String>, String> {
    let path = dataset_root.join("derived/manifest.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let m: ArtifactManifest = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if m.artifact_hash.is_empty() {
        return Ok(None);
    }
    Ok(Some(m.artifact_hash))
}

fn dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0_u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(p) = stack.pop() {
        if let Ok(rd) = fs::read_dir(&p) {
            for e in rd.flatten() {
                let p = e.path();
                if let Ok(md) = e.metadata() {
                    if md.is_dir() {
                        stack.push(p);
                    } else {
                        total = total.saturating_add(md.len());
                    }
                }
            }
        }
    }
    total
}

fn ensure_within_root(root: &Path, target: &Path) -> Result<(), String> {
    let c = target
        .canonicalize()
        .map_err(|e| format!("canonicalize {} failed: {e}", target.display()))?;
    if !c.starts_with(root) {
        return Err(format!(
            "refusing to touch path outside store root: {}",
            c.display()
        ));
    }
    Ok(())
}
