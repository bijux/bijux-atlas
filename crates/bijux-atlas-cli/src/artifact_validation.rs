use crate::{sha256_hex, OutputMode};
use bijux_atlas_core::canonical;
use bijux_atlas_model::{
    parse_dataset_key, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ReleaseGeneIndex,
    ShardCatalog,
};
use bijux_atlas_policies::{canonical_config_json, load_policy_from_workspace};
use bijux_atlas_store::{
    canonical_catalog_json, sorted_catalog_entries, verify_expected_sha256, ArtifactStore,
    LocalFsStore, ManifestLock, StoreErrorCode,
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder, Header};

pub(crate) fn parse_alias_map(input: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for pair in input.split(',') {
        let p = pair.trim();
        if p.is_empty() {
            continue;
        }
        if let Some((k, v)) = p.split_once('=') {
            out.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

pub(crate) fn validate_catalog(path: PathBuf, output_mode: OutputMode) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    let payload = json!({"command":"atlas catalog validate","status":"ok"});
    if output_mode.json {
        println!(
            "{}",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

pub(crate) fn validate_policy(output_mode: OutputMode) -> Result<(), String> {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let policy = load_policy_from_workspace(&workspace).map_err(|e| e.to_string())?;
    let canonical = canonical_config_json(&policy).map_err(|e| e.to_string())?;
    if output_mode.json {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "command":"atlas policy validate",
                "status":"ok",
                "schema_version": policy.schema_version.as_str(),
                "canonical": serde_json::from_str::<serde_json::Value>(&canonical).map_err(|e| e.to_string())?
            }))
            .map_err(|e| e.to_string())?
        );
    } else {
        println!("{canonical}");
    }
    Ok(())
}

pub(crate) fn publish_catalog(
    store_root: PathBuf,
    catalog_path: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let raw = fs::read_to_string(&catalog_path).map_err(|e| e.to_string())?;
    let mut catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.datasets = sorted_catalog_entries(catalog.datasets);
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    let canonical = canonical_catalog_json(&catalog)?;

    fs::create_dir_all(&store_root).map_err(|e| e.to_string())?;
    let tmp = store_root.join("catalog.json.tmp");
    fs::write(&tmp, canonical.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, store_root.join("catalog.json")).map_err(|e| e.to_string())?;

    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog publish","status":"ok"}),
    )
}

pub(crate) fn rollback_catalog(
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let path = store_root.join("catalog.json");
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let target = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    catalog.datasets.retain(|x| x.dataset != target);
    catalog.datasets = sorted_catalog_entries(catalog.datasets);
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    let canonical = canonical_catalog_json(&catalog)?;
    let tmp = store_root.join("catalog.json.tmp");
    fs::write(&tmp, canonical.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog rollback","status":"ok"}),
    )
}

pub(crate) fn promote_catalog(
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::artifact_paths(&store_root, &dataset);
    if !paths.manifest.exists() || !paths.sqlite.exists() {
        return Err(format!(
            "promote requires published artifact first: missing {} or {}",
            paths.manifest.display(),
            paths.sqlite.display()
        ));
    }

    let mut catalog = read_catalog_or_empty(&store_root)?;
    if !catalog.datasets.iter().any(|x| x.dataset == dataset) {
        catalog.datasets.push(CatalogEntry::new(
            dataset.clone(),
            rel_display_path(&store_root, &paths.manifest)?,
            rel_display_path(&store_root, &paths.sqlite)?,
        ));
    }
    catalog.datasets = sorted_catalog_entries(catalog.datasets);
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    write_catalog(&store_root, &catalog)?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog promote","status":"ok","dataset":dataset}),
    )
}

pub(crate) fn update_latest_alias(
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let catalog = read_catalog_or_empty(&store_root)?;
    if !catalog.datasets.iter().any(|x| x.dataset == dataset) {
        return Err(
            "latest alias update is gated by promotion: dataset not present in catalog".to_string(),
        );
    }
    fs::create_dir_all(&store_root).map_err(|e| e.to_string())?;
    let alias_path = store_root.join("latest.alias.json");
    let tmp = store_root.join("latest.alias.json.tmp");
    fs::write(
        &tmp,
        canonical::stable_json_bytes(&json!({
            "dataset": dataset,
            "policy": "promotion-gated"
        }))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&tmp, &alias_path).map_err(|e| e.to_string())?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog latest-alias-update","status":"ok","alias_path":alias_path}),
    )
}

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
struct GcReport {
    command: String,
    status: String,
    store_root: String,
    catalogs: Vec<String>,
    pins_path: String,
    reachable: ReachableSummary,
    candidates: CandidateSummary,
    applied: AppliedSummary,
    metrics: serde_json::Value,
    report_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ReachableSummary {
    dataset_count: usize,
    pinned_dataset_count: usize,
    pinned_hash_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CandidateSummary {
    dataset_roots: Vec<String>,
    bytes_by_root: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, serde::Serialize, Default)]
struct AppliedSummary {
    deleted_dataset_roots: Vec<String>,
    deleted_bytes: u64,
    errors: Vec<String>,
}

fn compute_gc_plan(
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

pub(crate) fn build_release_diff(
    root: PathBuf,
    from_release: &str,
    to_release: &str,
    species: &str,
    assembly: &str,
    out_dir: PathBuf,
    max_inline_items: usize,
    output_mode: OutputMode,
) -> Result<(), String> {
    let from = DatasetId::new(from_release, species, assembly).map_err(|e| e.to_string())?;
    let to = DatasetId::new(to_release, species, assembly).map_err(|e| e.to_string())?;
    let from_paths = bijux_atlas_model::artifact_paths(&root, &from);
    let to_paths = bijux_atlas_model::artifact_paths(&root, &to);
    let from_index = read_release_index(&from_paths.release_gene_index)?;
    let to_index = read_release_index(&to_paths.release_gene_index)?;
    let from_biotype = read_gene_biotypes(&from_paths.sqlite)?;
    let to_biotype = read_gene_biotypes(&to_paths.sqlite)?;

    let from_map = index_by_identity(&from_index.entries);
    let to_map = index_by_identity(&to_index.entries);
    let from_keys: HashSet<String> = from_map.keys().cloned().collect();
    let to_keys: HashSet<String> = to_map.keys().cloned().collect();

    let genes_added = sorted_strings(to_keys.difference(&from_keys).cloned().collect());
    let genes_removed = sorted_strings(from_keys.difference(&to_keys).cloned().collect());
    let mut genes_changed_coords = Vec::new();
    let mut genes_changed_biotype = Vec::new();
    let mut genes_changed_signature = Vec::new();

    let mut common = from_keys
        .intersection(&to_keys)
        .cloned()
        .collect::<Vec<_>>();
    common.sort();
    for key in common {
        let Some(left) = from_map.get(&key) else {
            continue;
        };
        let Some(right) = to_map.get(&key) else {
            continue;
        };
        if left.seqid != right.seqid || left.start != right.start || left.end != right.end {
            genes_changed_coords.push(key.clone());
        }
        let left_biotype = from_biotype.get(&left.gene_id).cloned().unwrap_or_default();
        let right_biotype = to_biotype.get(&right.gene_id).cloned().unwrap_or_default();
        if left_biotype != right_biotype {
            genes_changed_biotype.push(key.clone());
        }
        if left.signature_sha256 != right.signature_sha256 {
            genes_changed_signature.push(key);
        }
    }
    genes_changed_coords.sort();
    genes_changed_biotype.sort();
    genes_changed_signature.sort();

    let identity = json!({
        "from_release": from_release,
        "to_release": to_release,
        "species": species,
        "assembly": assembly
    });
    let summary = json!({
        "schema_version": "1",
        "identity": identity,
        "counts": {
            "genes_added": genes_added.len(),
            "genes_removed": genes_removed.len(),
            "genes_changed_coords": genes_changed_coords.len(),
            "genes_changed_biotype": genes_changed_biotype.len(),
            "genes_changed_signature": genes_changed_signature.len()
        },
        "sanity": {
            "stable_gene_ratio": stable_ratio(from_map.len(), genes_added.len(), genes_removed.len())
        }
    });

    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let mut chunk_manifest = Vec::<serde_json::Value>::new();
    let genes_added = chunk_or_inline(
        "genes_added",
        genes_added,
        max_inline_items,
        &out_dir,
        &mut chunk_manifest,
    )?;
    let genes_removed = chunk_or_inline(
        "genes_removed",
        genes_removed,
        max_inline_items,
        &out_dir,
        &mut chunk_manifest,
    )?;
    let genes_changed_coords = chunk_or_inline(
        "genes_changed_coords",
        genes_changed_coords,
        max_inline_items,
        &out_dir,
        &mut chunk_manifest,
    )?;
    let genes_changed_biotype = chunk_or_inline(
        "genes_changed_biotype",
        genes_changed_biotype,
        max_inline_items,
        &out_dir,
        &mut chunk_manifest,
    )?;
    let genes_changed_signature = chunk_or_inline(
        "genes_changed_signature",
        genes_changed_signature,
        max_inline_items,
        &out_dir,
        &mut chunk_manifest,
    )?;

    let diff = json!({
        "schema_version": "1",
        "identity": identity,
        "genes_added": genes_added,
        "genes_removed": genes_removed,
        "genes_changed_coords": genes_changed_coords,
        "genes_changed_biotype": genes_changed_biotype,
        "genes_changed_signature": genes_changed_signature,
        "transcripts_added": [],
        "transcripts_removed": [],
        "chunk_manifest": chunk_manifest,
        "compatibility": "additive-only"
    });

    let diff_path = out_dir.join("diff.json");
    let summary_path = out_dir.join("diff.summary.json");
    let diff_bytes = canonical::stable_json_bytes(&diff).map_err(|e| e.to_string())?;
    let summary_bytes = canonical::stable_json_bytes(&summary).map_err(|e| e.to_string())?;
    fs::write(&diff_path, &diff_bytes).map_err(|e| e.to_string())?;
    fs::write(&summary_path, &summary_bytes).map_err(|e| e.to_string())?;
    let diff_sha = sha256_hex(&diff_bytes);
    emit_ok_payload(
        output_mode,
        json!({
            "command":"atlas diff build",
            "status":"ok",
            "schema_version":"1",
            "identity": diff["identity"],
            "diff_path": diff_path,
            "summary_path": summary_path,
            "diff_sha256": diff_sha
        }),
    )
}

fn read_release_index(path: &Path) -> Result<ReleaseGeneIndex, String> {
    let raw = fs::read(path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    serde_json::from_slice(&raw).map_err(|e| format!("parse {} failed: {e}", path.display()))
}

fn read_gene_biotypes(sqlite: &Path) -> Result<HashMap<String, String>, String> {
    let conn = rusqlite::Connection::open(sqlite).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT gene_id, biotype FROM gene_summary")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut out = HashMap::new();
    for row in rows {
        let (gene_id, biotype) = row.map_err(|e| e.to_string())?;
        out.insert(gene_id, biotype);
    }
    Ok(out)
}

fn index_by_identity(
    entries: &[bijux_atlas_model::ReleaseGeneIndexEntry],
) -> HashMap<String, bijux_atlas_model::ReleaseGeneIndexEntry> {
    let mut out = HashMap::with_capacity(entries.len());
    for e in entries {
        out.insert(stable_gene_identity(e), e.clone());
    }
    out
}

fn stable_gene_identity(entry: &bijux_atlas_model::ReleaseGeneIndexEntry) -> String {
    if !entry.gene_id.trim().is_empty() {
        return entry.gene_id.clone();
    }
    format!("{}:{}-{}", entry.seqid, entry.start, entry.end)
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn stable_ratio(total_from: usize, added: usize, removed: usize) -> f64 {
    if total_from == 0 {
        return 1.0;
    }
    let stable = total_from
        .saturating_sub(removed)
        .saturating_sub(added.min(total_from));
    (stable as f64) / (total_from as f64)
}

fn chunk_or_inline(
    name: &str,
    mut rows: Vec<String>,
    max_inline_items: usize,
    out_dir: &Path,
    chunk_manifest: &mut Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    rows.sort();
    if rows.len() <= max_inline_items {
        return Ok(serde_json::Value::Array(
            rows.into_iter().map(serde_json::Value::String).collect(),
        ));
    }
    let chunks_dir = out_dir.join("chunks");
    fs::create_dir_all(&chunks_dir).map_err(|e| e.to_string())?;
    for (idx, chunk) in rows.chunks(max_inline_items).enumerate() {
        let path = chunks_dir.join(format!("{name}.{:03}.json", idx));
        let payload = serde_json::Value::Array(
            chunk
                .iter()
                .cloned()
                .map(serde_json::Value::String)
                .collect(),
        );
        let bytes = canonical::stable_json_bytes(&payload).map_err(|e| e.to_string())?;
        fs::write(&path, bytes).map_err(|e| e.to_string())?;
        chunk_manifest.push(json!({
            "field": name,
            "chunk": idx,
            "path": format!("chunks/{}.{}.json", name, format!("{idx:03}")),
            "count": chunk.len()
        }));
    }
    Ok(json!({
        "truncated": true,
        "total_count": rows.len(),
        "inline_count": 0
    }))
}

fn read_catalog_or_empty(store_root: &Path) -> Result<Catalog, String> {
    let path = store_root.join("catalog.json");
    if !path.exists() {
        return Ok(Catalog::new(Vec::new()));
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(catalog)
}

fn write_catalog(store_root: &Path, catalog: &Catalog) -> Result<(), String> {
    let canonical = canonical_catalog_json(catalog)?;
    fs::create_dir_all(store_root).map_err(|e| e.to_string())?;
    let tmp = store_root.join("catalog.json.tmp");
    fs::write(&tmp, canonical.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, store_root.join("catalog.json")).map_err(|e| e.to_string())
}

fn rel_display_path(root: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| format!("path {} is outside {}", path.display(), root.display()))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn validate_dataset(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    deep: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::artifact_paths(&root, &dataset);

    let manifest_raw = fs::read_to_string(&paths.manifest).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    manifest.validate_strict().map_err(|e| e.to_string())?;

    check_sha(&paths.gff3, &manifest.checksums.gff3_sha256)?;
    check_sha(&paths.fasta, &manifest.checksums.fasta_sha256)?;
    check_sha(&paths.fai, &manifest.checksums.fai_sha256)?;
    check_sha(&paths.sqlite, &manifest.checksums.sqlite_sha256)?;

    let sqlite_bytes = fs::read(&paths.sqlite).map_err(|e| e.to_string())?;
    if !sqlite_bytes.starts_with(b"SQLite format 3\0") {
        return Err("sqlite artifact does not start with SQLite header".to_string());
    }

    if manifest.stats.gene_count == 0 {
        return Err("manifest gene_count must be > 0".to_string());
    }
    validate_sqlite_contract(&paths.sqlite)?;
    validate_shard_catalog_and_indexes(&paths.derived_dir)?;
    if deep {
        let lock_path = paths.derived_dir.join("manifest.lock");
        let lock_raw = fs::read(&lock_path)
            .map_err(|_| format!("manifest.lock missing: {}", lock_path.display()))?;
        let lock: ManifestLock = serde_json::from_slice(&lock_raw).map_err(|e| e.to_string())?;
        lock.validate(manifest_raw.as_bytes(), &sqlite_bytes)?;

        let actual_signature = compute_dataset_signature_from_sqlite(&paths.sqlite)?;
        if manifest.dataset_signature_sha256.is_empty() {
            return Err(
                "manifest dataset_signature_sha256 is empty; cannot deep-verify".to_string(),
            );
        }
        if actual_signature != manifest.dataset_signature_sha256 {
            return Err(format!(
                "dataset signature mismatch: manifest={} actual={}",
                manifest.dataset_signature_sha256, actual_signature
            ));
        }
        if manifest.derived_column_origins.is_empty() {
            return Err("manifest derived_column_origins must not be empty".to_string());
        }
        enforce_publish_gates(&root, &dataset, &manifest)?;
    }

    let command_name = if deep {
        "atlas dataset verify"
    } else {
        "atlas dataset validate"
    };
    let payload = json!({"command":command_name,"status":"ok","deep":deep});
    if output_mode.json {
        println!(
            "{}",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

pub(crate) fn validate_ingest_qc(
    qc_report: PathBuf,
    thresholds: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let qc_raw = fs::read_to_string(&qc_report)
        .map_err(|e| format!("failed to read {}: {e}", qc_report.display()))?;
    let thresholds_raw = fs::read_to_string(&thresholds)
        .map_err(|e| format!("failed to read {}: {e}", thresholds.display()))?;
    let qc: serde_json::Value = serde_json::from_str(&qc_raw)
        .map_err(|e| format!("invalid QC json {}: {e}", qc_report.display()))?;
    let t: serde_json::Value = serde_json::from_str(&thresholds_raw)
        .map_err(|e| format!("invalid thresholds json {}: {e}", thresholds.display()))?;
    validate_qc_thresholds(&qc, &t)?;
    emit_ok_payload(
        output_mode,
        json!({
            "command":"atlas ingest-validate",
            "status":"ok",
            "qc_report": qc_report,
            "thresholds": thresholds
        }),
    )
}

fn compute_dataset_signature_from_sqlite(sqlite_path: &PathBuf) -> Result<String, String> {
    let conn = rusqlite::Connection::open(sqlite_path).map_err(|e| e.to_string())?;
    let mut gene_stmt = conn
        .prepare(
            "SELECT gene_id, name, biotype, seqid, start, end, transcript_count, exon_count, total_exon_span, cds_present, sequence_length
             FROM gene_summary ORDER BY seqid, start, end, gene_id",
        )
        .map_err(|e| e.to_string())?;
    let genes = gene_stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "gene_id": r.get::<_, String>(0)?,
                "gene_name": r.get::<_, String>(1)?,
                "biotype": r.get::<_, String>(2)?,
                "seqid": r.get::<_, String>(3)?,
                "start": r.get::<_, i64>(4)?,
                "end": r.get::<_, i64>(5)?,
                "transcript_count": r.get::<_, i64>(6)?,
                "exon_count": r.get::<_, i64>(7)?,
                "total_exon_span": r.get::<_, i64>(8)?,
                "cds_present": r.get::<_, i64>(9)? != 0,
                "sequence_length": r.get::<_, i64>(10)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut tx_stmt = conn
        .prepare(
            "SELECT transcript_id, parent_gene_id, transcript_type, COALESCE(biotype,''), seqid, start, end, exon_count, total_exon_span, cds_present
             FROM transcript_summary ORDER BY seqid, start, end, transcript_id",
        )
        .map_err(|e| e.to_string())?;
    let txs = tx_stmt
        .query_map([], |r| {
            let raw_biotype: String = r.get(3)?;
            let biotype = if raw_biotype.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(raw_biotype)
            };
            Ok(serde_json::json!({
                "transcript_id": r.get::<_, String>(0)?,
                "parent_gene_id": r.get::<_, String>(1)?,
                "transcript_type": r.get::<_, String>(2)?,
                "biotype": biotype,
                "seqid": r.get::<_, String>(4)?,
                "start": r.get::<_, i64>(5)?,
                "end": r.get::<_, i64>(6)?,
                "exon_count": r.get::<_, i64>(7)?,
                "total_exon_span": r.get::<_, i64>(8)?,
                "cds_present": r.get::<_, i64>(9)? != 0,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let root = serde_json::json!({
        "gene_table_hash": merkle_from_json_rows(&genes)?,
        "transcript_table_hash": merkle_from_json_rows(&txs)?,
        "gene_count": genes.len(),
        "transcript_count": txs.len(),
    });
    let bytes = canonical::stable_json_bytes(&root).map_err(|e| e.to_string())?;
    Ok(sha256_hex(&bytes))
}

fn merkle_from_json_rows(rows: &[serde_json::Value]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(sha256_hex(b""));
    }
    let mut level: Vec<String> = rows
        .iter()
        .map(|r| canonical::stable_json_bytes(r).map(|b| sha256_hex(&b)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        let mut i = 0usize;
        while i < level.len() {
            let left = &level[i];
            let right = if i + 1 < level.len() {
                &level[i + 1]
            } else {
                left
            };
            let mut joined = String::with_capacity(left.len() + right.len());
            joined.push_str(left);
            joined.push_str(right);
            next.push(sha256_hex(joined.as_bytes()));
            i += 2;
        }
        level = next;
    }
    Ok(level[0].clone())
}

pub(crate) fn publish_dataset(
    source_root: PathBuf,
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let source_paths = bijux_atlas_model::artifact_paths(&source_root, &dataset);
    let manifest_bytes = fs::read(&source_paths.manifest).map_err(|e| e.to_string())?;
    let sqlite_bytes = fs::read(&source_paths.sqlite).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|e| e.to_string())?;
    manifest.validate_strict().map_err(|e| e.to_string())?;
    verify_expected_sha256(&sqlite_bytes, &manifest.checksums.sqlite_sha256)?;
    enforce_publish_gates(&source_root, &dataset, &manifest)?;
    let manifest_sha = sha256_hex(&manifest_bytes);
    let sqlite_sha = sha256_hex(&sqlite_bytes);

    let store = LocalFsStore::new(store_root);
    match store.put_dataset(
        &dataset,
        &manifest_bytes,
        &sqlite_bytes,
        &manifest_sha,
        &sqlite_sha,
    ) {
        Ok(()) => emit_ok_payload(
            output_mode,
            json!({"command":"atlas dataset publish","status":"ok"}),
        ),
        Err(e) if e.code == StoreErrorCode::Conflict => {
            Err(format!("immutability gate rejected publish: {}", e.message))
        }
        Err(e) => Err(e.to_string()),
    }
}

fn enforce_publish_gates(
    source_root: &Path,
    dataset: &DatasetId,
    manifest: &ArtifactManifest,
) -> Result<(), String> {
    let workspace = std::env::current_dir().map_err(|e| e.to_string())?;
    let policy = load_policy_from_workspace(&workspace).map_err(|e| e.to_string())?;
    if manifest.stats.gene_count < policy.publish_gates.min_gene_count {
        return Err(format!(
            "publish gate failed: gene_count {} < min_gene_count {}",
            manifest.stats.gene_count, policy.publish_gates.min_gene_count
        ));
    }
    let paths = bijux_atlas_model::artifact_paths(source_root, dataset);
    let anomaly_raw = fs::read_to_string(paths.anomaly_report).map_err(|e| e.to_string())?;
    let anomaly: bijux_atlas_model::IngestAnomalyReport =
        serde_json::from_str(&anomaly_raw).map_err(|e| e.to_string())?;
    if (anomaly.missing_parents.len() as u64) > policy.publish_gates.max_missing_parents {
        return Err(format!(
            "publish gate failed: missing_parents {} > max_missing_parents {}",
            anomaly.missing_parents.len(),
            policy.publish_gates.max_missing_parents
        ));
    }
    let conn = rusqlite::Connection::open(paths.sqlite).map_err(|e| e.to_string())?;
    for idx in &policy.publish_gates.required_indexes {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                [idx],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err(format!(
                "publish gate failed: required index missing: {idx}"
            ));
        }
    }
    let qc_report = paths.derived_dir.join("qc.json");
    let thresholds_path = workspace.join("configs/ops/dataset-qc-thresholds.json");
    let qc_raw = fs::read_to_string(&qc_report)
        .map_err(|e| format!("publish gate failed: {}: {e}", qc_report.display()))?;
    let thresholds_raw = fs::read_to_string(&thresholds_path)
        .map_err(|e| format!("publish gate failed: {}: {e}", thresholds_path.display()))?;
    let qc: serde_json::Value = serde_json::from_str(&qc_raw).map_err(|e| {
        format!(
            "publish gate failed: invalid qc json {}: {e}",
            qc_report.display()
        )
    })?;
    let thresholds: serde_json::Value = serde_json::from_str(&thresholds_raw).map_err(|e| {
        format!(
            "publish gate failed: invalid thresholds json {}: {e}",
            thresholds_path.display()
        )
    })?;
    validate_qc_thresholds(&qc, &thresholds).map_err(|e| format!("publish gate failed: {e}"))?;
    Ok(())
}

fn validate_qc_thresholds(
    qc: &serde_json::Value,
    thresholds: &serde_json::Value,
) -> Result<(), String> {
    let min_gene_count = thresholds
        .get("min_gene_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "threshold missing min_gene_count".to_string())?;
    let max_orphan_pct = thresholds
        .get("max_orphan_percent")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "threshold missing max_orphan_percent".to_string())?;
    let max_rejected_pct = thresholds
        .get("max_rejected_percent")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "threshold missing max_rejected_percent".to_string())?;
    let max_unknown_contig_pct = thresholds
        .get("max_unknown_contig_feature_percent")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "threshold missing max_unknown_contig_feature_percent".to_string())?;
    let max_duplicate_gene_id_events = thresholds
        .get("max_duplicate_gene_id_events")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "threshold missing max_duplicate_gene_id_events".to_string())?;
    let genes = qc
        .pointer("/counts/genes")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing counts.genes".to_string())?;
    let transcripts = qc
        .pointer("/counts/transcripts")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing counts.transcripts".to_string())?;
    let orphan_transcripts = qc
        .pointer("/orphan_counts/transcripts")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing orphan_counts.transcripts".to_string())?;
    let duplicate_gene_ids = qc
        .pointer("/duplicate_id_events/duplicate_gene_ids")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing duplicate_id_events.duplicate_gene_ids".to_string())?;
    let unknown_contig_ratio = qc
        .pointer("/contig_stats/unknown_contig_feature_ratio")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "qc missing contig_stats.unknown_contig_feature_ratio".to_string())?;
    let rejected: u64 = qc
        .get("rejected_record_count_by_reason")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "qc missing rejected_record_count_by_reason".to_string())?
        .values()
        .map(|v| v.as_u64().unwrap_or(0))
        .sum();
    if genes < min_gene_count {
        return Err(format!(
            "gene_count {} < min_gene_count {}",
            genes, min_gene_count
        ));
    }
    let orphan_pct = if transcripts == 0 {
        0.0
    } else {
        (orphan_transcripts as f64) * 100.0 / (transcripts as f64)
    };
    if orphan_pct > max_orphan_pct {
        return Err(format!(
            "orphan_percent {:.4} > max_orphan_percent {}",
            orphan_pct, max_orphan_pct
        ));
    }
    let total_features = qc
        .pointer("/contig_stats/total_features")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing contig_stats.total_features".to_string())?;
    let rejected_pct = if total_features == 0 {
        0.0
    } else {
        (rejected as f64) * 100.0 / (total_features as f64)
    };
    if rejected_pct > max_rejected_pct {
        return Err(format!(
            "rejected_percent {:.4} > max_rejected_percent {}",
            rejected_pct, max_rejected_pct
        ));
    }
    if unknown_contig_ratio * 100.0 > max_unknown_contig_pct {
        return Err(format!(
            "unknown_contig_feature_percent {:.4} > max_unknown_contig_feature_percent {}",
            unknown_contig_ratio * 100.0,
            max_unknown_contig_pct
        ));
    }
    if duplicate_gene_ids > max_duplicate_gene_id_events {
        return Err(format!(
            "duplicate_gene_id_events {} > max_duplicate_gene_id_events {}",
            duplicate_gene_ids, max_duplicate_gene_id_events
        ));
    }
    Ok(())
}

pub(crate) fn pack_dataset(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    out: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::artifact_paths(&root, &dataset);
    let manifest = fs::read(&paths.manifest).map_err(|e| e.to_string())?;
    let sqlite = fs::read(&paths.sqlite).map_err(|e| e.to_string())?;
    let lock = ManifestLock::from_bytes(&manifest, &sqlite);
    let lock_bytes = serde_json::to_vec(&lock).map_err(|e| e.to_string())?;

    let file = fs::File::create(&out).map_err(|e| e.to_string())?;
    let mut builder = Builder::new(file);
    append_tar_file(&mut builder, "manifest.json", &manifest)?;
    append_tar_file(&mut builder, "gene_summary.sqlite", &sqlite)?;
    append_tar_file(&mut builder, "manifest.lock", &lock_bytes)?;
    builder.finish().map_err(|e| e.to_string())?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas dataset pack","status":"ok","out":out}),
    )
}

pub(crate) fn verify_pack(pack: PathBuf, output_mode: OutputMode) -> Result<(), String> {
    let file = fs::File::open(pack).map_err(|e| e.to_string())?;
    let mut archive = Archive::new(file);
    let mut manifest: Option<Vec<u8>> = None;
    let mut sqlite: Option<Vec<u8>> = None;
    let mut lock_raw: Option<Vec<u8>> = None;
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut e = entry.map_err(|e| e.to_string())?;
        let path = e
            .path()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut e, &mut bytes).map_err(|e| e.to_string())?;
        match path.as_str() {
            "manifest.json" => manifest = Some(bytes),
            "gene_summary.sqlite" => sqlite = Some(bytes),
            "manifest.lock" => lock_raw = Some(bytes),
            _ => {}
        }
    }
    let manifest = manifest.ok_or_else(|| "manifest.json missing in pack".to_string())?;
    let sqlite = sqlite.ok_or_else(|| "gene_summary.sqlite missing in pack".to_string())?;
    let lock_raw = lock_raw.ok_or_else(|| "manifest.lock missing in pack".to_string())?;
    let lock: ManifestLock = serde_json::from_slice(&lock_raw).map_err(|e| e.to_string())?;
    lock.validate(&manifest, &sqlite)?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas dataset verify-pack","status":"ok"}),
    )
}

fn append_tar_file(
    builder: &mut Builder<std::fs::File>,
    name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_cksum();
    builder
        .append_data(&mut header, name, bytes)
        .map_err(|e| e.to_string())
}

fn emit_ok_payload(output_mode: OutputMode, payload: serde_json::Value) -> Result<(), String> {
    if output_mode.json {
        println!(
            "{}",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

fn validate_sqlite_contract(sqlite_path: &PathBuf) -> Result<(), String> {
    let conn = rusqlite::Connection::open(sqlite_path).map_err(|e| e.to_string())?;
    let required_indexes = [
        "idx_gene_summary_gene_id",
        "idx_gene_summary_name",
        "idx_gene_summary_name_normalized",
        "idx_gene_summary_biotype",
        "idx_gene_summary_region",
        "idx_gene_summary_cover_lookup",
        "idx_gene_summary_cover_region",
        "idx_transcript_summary_transcript_id",
        "idx_transcript_summary_parent_gene_id",
        "idx_transcript_summary_biotype",
        "idx_transcript_summary_type",
        "idx_transcript_summary_region",
    ];
    for index in required_indexes {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                [index],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err(format!("required index missing: {index}"));
        }
    }
    let has_rtree: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='gene_summary_rtree'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_rtree == 0 {
        return Err("required rtree table missing: gene_summary_rtree".to_string());
    }
    let has_transcript_table: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transcript_summary'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_transcript_table == 0 {
        return Err("required table missing: transcript_summary".to_string());
    }
    let schema_version = read_schema_version(&conn)?;
    if schema_version <= 0 {
        return Err("schema_version must be positive".to_string());
    }
    let analyzed: String = conn
        .query_row(
            "SELECT v FROM atlas_meta WHERE k='analyze_completed'",
            [],
            |r| r.get(0),
        )
        .map_err(|_| "atlas_meta.analyze_completed missing".to_string())?;
    if analyzed != "true" {
        return Err("ANALYZE required gate failed: analyze_completed != true".to_string());
    }
    Ok(())
}

fn read_schema_version(conn: &rusqlite::Connection) -> Result<i64, String> {
    let has_schema_table: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_schema_table > 0 {
        return conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string());
    }
    let legacy_schema_version: String = conn
        .query_row(
            "SELECT v FROM atlas_meta WHERE k='schema_version'",
            [],
            |r| r.get(0),
        )
        .map_err(|_| "atlas_meta.schema_version missing".to_string())?;
    legacy_schema_version
        .parse::<i64>()
        .map_err(|_| format!("invalid atlas_meta.schema_version: {legacy_schema_version}"))
}

fn validate_shard_catalog_and_indexes(derived_dir: &std::path::Path) -> Result<(), String> {
    let path = derived_dir.join("catalog_shards.json");
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let catalog: ShardCatalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    for shard in &catalog.shards {
        let shard_path = derived_dir.join(&shard.sqlite_path);
        validate_sqlite_contract(&shard_path)?;
        let bytes = fs::read(&shard_path).map_err(|e| e.to_string())?;
        let actual = sha256_hex(&bytes);
        if actual != shard.sqlite_sha256 {
            return Err(format!(
                "shard checksum mismatch for {}",
                shard_path.display()
            ));
        }
    }
    Ok(())
}

fn check_sha(path: &PathBuf, expected: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!(
            "checksum mismatch for {}: expected {}, got {}",
            path.display(),
            expected,
            actual
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_release_diff, compute_gc_plan, validate_qc_thresholds, OutputMode};
    use bijux_atlas_core::sha256_hex;
    use bijux_atlas_model::{Catalog, CatalogEntry, DatasetId};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn qc_thresholds_pass_for_healthy_report() {
        let qc = json!({
            "counts": {"genes": 10, "transcripts": 20, "exons": 50, "cds": 12},
            "orphan_counts": {"transcripts": 0},
            "duplicate_id_events": {"duplicate_gene_ids": 0},
            "rejected_record_count_by_reason": {"GFF3_UNKNOWN_FEATURE": 0},
            "contig_stats": {"unknown_contig_feature_ratio": 0.0, "total_features": 100}
        });
        let t = json!({
            "min_gene_count": 1,
            "max_orphan_percent": 1.0,
            "max_rejected_percent": 1.0,
            "max_unknown_contig_feature_percent": 0.5,
            "max_duplicate_gene_id_events": 0
        });
        assert!(validate_qc_thresholds(&qc, &t).is_ok());
    }

    #[test]
    fn qc_thresholds_fail_when_orphan_rate_exceeds_max() {
        let qc = json!({
            "counts": {"genes": 10, "transcripts": 10, "exons": 10, "cds": 10},
            "orphan_counts": {"transcripts": 2},
            "duplicate_id_events": {"duplicate_gene_ids": 0},
            "rejected_record_count_by_reason": {},
            "contig_stats": {"unknown_contig_feature_ratio": 0.0, "total_features": 100}
        });
        let t = json!({
            "min_gene_count": 1,
            "max_orphan_percent": 10.0,
            "max_rejected_percent": 10.0,
            "max_unknown_contig_feature_percent": 10.0,
            "max_duplicate_gene_id_events": 0
        });
        let err = validate_qc_thresholds(&qc, &t).expect_err("orphan gate must fail");
        assert!(err.contains("orphan_percent"));
    }

    #[test]
    fn qc_edgecase_fixture_orphan_rate_regression_is_rejected() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let qc = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(root.join("tests/fixtures/qc_edgecases/qc_orphan_high.json"))
                .expect("read qc fixture"),
        )
        .expect("parse qc fixture");
        let t = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(root.join("tests/fixtures/qc_edgecases/thresholds_strict.json"))
                .expect("read threshold fixture"),
        )
        .expect("parse threshold fixture");
        assert!(validate_qc_thresholds(&qc, &t).is_err());
    }

    #[test]
    fn diff_build_is_deterministic_for_same_inputs() {
        let tmp = tempdir().expect("tmp");
        let root = tmp.path();
        let from = root.join("release=110/species=homo_sapiens/assembly=GRCh38/derived");
        let to = root.join("release=111/species=homo_sapiens/assembly=GRCh38/derived");
        fs::create_dir_all(&from).expect("create from");
        fs::create_dir_all(&to).expect("create to");

        fs::write(
            from.join("release_gene_index.json"),
            r#"{"schema_version":"1","dataset":{"release":"110","species":"homo_sapiens","assembly":"GRCh38"},"entries":[{"gene_id":"ENSG1","seqid":"chr1","start":10,"end":20,"signature_sha256":"a"},{"gene_id":"ENSG2","seqid":"chr1","start":30,"end":40,"signature_sha256":"b"}]}"#,
        )
        .expect("write from index");
        fs::write(
            to.join("release_gene_index.json"),
            r#"{"schema_version":"1","dataset":{"release":"111","species":"homo_sapiens","assembly":"GRCh38"},"entries":[{"gene_id":"ENSG1","seqid":"chr1","start":10,"end":25,"signature_sha256":"z"},{"gene_id":"ENSG3","seqid":"chr2","start":5,"end":8,"signature_sha256":"c"}]}"#,
        )
        .expect("write to index");

        write_sqlite(
            &from.join("gene_summary.sqlite"),
            &[("ENSG1", "protein_coding"), ("ENSG2", "lncRNA")],
        );
        write_sqlite(
            &to.join("gene_summary.sqlite"),
            &[("ENSG1", "protein_coding"), ("ENSG3", "miRNA")],
        );

        let out1 = root.join("diff-out-1");
        let out2 = root.join("diff-out-2");
        build_release_diff(
            root.to_path_buf(),
            "110",
            "111",
            "homo_sapiens",
            "GRCh38",
            out1.clone(),
            100,
            OutputMode { json: true },
        )
        .expect("build diff #1");
        build_release_diff(
            root.to_path_buf(),
            "110",
            "111",
            "homo_sapiens",
            "GRCh38",
            out2.clone(),
            100,
            OutputMode { json: true },
        )
        .expect("build diff #2");
        let d1 = fs::read(out1.join("diff.json")).expect("read diff1");
        let d2 = fs::read(out2.join("diff.json")).expect("read diff2");
        assert_eq!(d1, d2, "diff output must be byte-identical");
        assert!(!sha256_hex(&d1).is_empty());
    }

    fn write_sqlite(path: &std::path::Path, rows: &[(&str, &str)]) {
        let conn = rusqlite::Connection::open(path).expect("open sqlite");
        conn.execute(
            "CREATE TABLE gene_summary (gene_id TEXT NOT NULL, biotype TEXT NOT NULL)",
            [],
        )
        .expect("create table");
        for (gene_id, biotype) in rows {
            conn.execute(
                "INSERT INTO gene_summary(gene_id, biotype) VALUES (?1, ?2)",
                [gene_id, biotype],
            )
            .expect("insert");
        }
    }

    #[test]
    fn gc_plan_respects_catalog_and_dataset_pins() {
        let tmp = tempdir().expect("tmp");
        let root = tmp.path().join("store");
        fs::create_dir_all(&root).expect("mkdir");

        let reachable = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
        let pinned = DatasetId::new("111", "homo_sapiens", "GRCh38").expect("dataset");
        let candidate = DatasetId::new("112", "homo_sapiens", "GRCh38").expect("dataset");
        for d in [&reachable, &pinned, &candidate] {
            let p = bijux_atlas_model::artifact_paths(&root, d);
            fs::create_dir_all(p.sqlite.parent().expect("parent")).expect("mkdir derived");
            fs::write(&p.sqlite, b"sqlite").expect("write sqlite");
        }

        let catalog = Catalog::new(vec![CatalogEntry::new(
            reachable.clone(),
            bijux_atlas_model::artifact_paths(&root, &reachable)
                .manifest
                .strip_prefix(&root)
                .expect("strip")
                .display()
                .to_string(),
            bijux_atlas_model::artifact_paths(&root, &reachable)
                .sqlite
                .strip_prefix(&root)
                .expect("strip")
                .display()
                .to_string(),
        )]);
        let catalog_path = root.join("catalog.json");
        fs::write(
            &catalog_path,
            bijux_atlas_store::canonical_catalog_json(&catalog).expect("catalog json"),
        )
        .expect("write catalog");
        let pins_path = tmp.path().join("pins.json");
        fs::write(
            &pins_path,
            serde_json::to_vec(&json!({
                "dataset_ids":[format!(
                    "release={}&species={}&assembly={}",
                    pinned.release, pinned.species, pinned.assembly
                )],
                "artifact_hashes":[]
            }))
            .expect("pins json"),
        )
        .expect("write pins");

        let report = compute_gc_plan(&root, &[catalog_path], &pins_path).expect("gc plan");
        assert_eq!(report.candidates.dataset_roots.len(), 1);
        let expected_paths = bijux_atlas_model::artifact_paths(&root, &candidate);
        let expected_root = expected_paths
            .manifest
            .parent()
            .and_then(|p| p.parent())
            .expect("dataset root");
        let actual_root = PathBuf::from(&report.candidates.dataset_roots[0])
            .canonicalize()
            .expect("canonical candidate");
        let expected_root = expected_root.canonicalize().expect("canonical expected");
        assert_eq!(actual_root, expected_root);
    }

    #[test]
    fn gc_plan_multiple_catalog_paths_are_deterministic() {
        let tmp = tempdir().expect("tmp");
        let root = tmp.path().join("store");
        fs::create_dir_all(&root).expect("mkdir");

        let d1 = DatasetId::new("200", "homo_sapiens", "GRCh38").expect("dataset");
        let d2 = DatasetId::new("201", "homo_sapiens", "GRCh38").expect("dataset");
        for d in [&d1, &d2] {
            let p = bijux_atlas_model::artifact_paths(&root, d);
            fs::create_dir_all(p.sqlite.parent().expect("parent")).expect("mkdir derived");
            fs::write(&p.sqlite, b"sqlite").expect("write sqlite");
        }

        let write_catalog = |name: &str, dataset: &DatasetId| -> PathBuf {
            let c = Catalog::new(vec![CatalogEntry::new(
                dataset.clone(),
                bijux_atlas_model::artifact_paths(&root, dataset)
                    .manifest
                    .strip_prefix(&root)
                    .expect("strip")
                    .display()
                    .to_string(),
                bijux_atlas_model::artifact_paths(&root, dataset)
                    .sqlite
                    .strip_prefix(&root)
                    .expect("strip")
                    .display()
                    .to_string(),
            )]);
            let path = root.join(name);
            fs::write(
                &path,
                bijux_atlas_store::canonical_catalog_json(&c).expect("catalog json"),
            )
            .expect("write catalog");
            path
        };
        let c1 = write_catalog("catalog-a.json", &d1);
        let c2 = write_catalog("catalog-b.json", &d2);
        let pins_path = tmp.path().join("pins.json");
        fs::write(&pins_path, br#"{"dataset_ids":[],"artifact_hashes":[]}"#).expect("pins");

        let r1 =
            compute_gc_plan(&root, &[c2.clone(), c1.clone(), c2.clone()], &pins_path).expect("gc");
        let r2 = compute_gc_plan(&root, &[c1, c2], &pins_path).expect("gc");
        assert_eq!(r1.catalogs, r2.catalogs);
        assert_eq!(r1.candidates.dataset_roots, r2.candidates.dataset_roots);
    }
}
