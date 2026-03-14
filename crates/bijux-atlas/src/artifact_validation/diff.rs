// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(crate) struct BuildReleaseDiffArgs {
    pub root: PathBuf,
    pub from_release: String,
    pub to_release: String,
    pub species: String,
    pub assembly: String,
    pub out_dir: PathBuf,
    pub max_inline_items: usize,
}

pub(crate) fn build_release_diff(
    args: BuildReleaseDiffArgs,
    output_mode: OutputMode,
) -> Result<(), String> {
    let from = DatasetId::new(&args.from_release, &args.species, &args.assembly)
        .map_err(|e| e.to_string())?;
    let to = DatasetId::new(&args.to_release, &args.species, &args.assembly)
        .map_err(|e| e.to_string())?;
    let from_paths = bijux_atlas_model::artifact_paths(&args.root, &from);
    let to_paths = bijux_atlas_model::artifact_paths(&args.root, &to);
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
        let left_biotype = from_biotype
            .get(left.gene_id.as_str())
            .cloned()
            .unwrap_or_default();
        let right_biotype = to_biotype
            .get(right.gene_id.as_str())
            .cloned()
            .unwrap_or_default();
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
        "from_release": args.from_release,
        "to_release": args.to_release,
        "species": args.species,
        "assembly": args.assembly
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

    fs::create_dir_all(&args.out_dir).map_err(|e| e.to_string())?;
    let mut chunk_manifest = Vec::<serde_json::Value>::new();
    let genes_added = chunk_or_inline(
        "genes_added",
        genes_added,
        args.max_inline_items,
        &args.out_dir,
        &mut chunk_manifest,
    )?;
    let genes_removed = chunk_or_inline(
        "genes_removed",
        genes_removed,
        args.max_inline_items,
        &args.out_dir,
        &mut chunk_manifest,
    )?;
    let genes_changed_coords = chunk_or_inline(
        "genes_changed_coords",
        genes_changed_coords,
        args.max_inline_items,
        &args.out_dir,
        &mut chunk_manifest,
    )?;
    let genes_changed_biotype = chunk_or_inline(
        "genes_changed_biotype",
        genes_changed_biotype,
        args.max_inline_items,
        &args.out_dir,
        &mut chunk_manifest,
    )?;
    let genes_changed_signature = chunk_or_inline(
        "genes_changed_signature",
        genes_changed_signature,
        args.max_inline_items,
        &args.out_dir,
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

    let diff_path = args.out_dir.join("diff.json");
    let summary_path = args.out_dir.join("diff.summary.json");
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
    if !entry.gene_id.as_str().trim().is_empty() {
        return entry.gene_id.as_str().to_string();
    }
    format!("{}:{}-{}", entry.seqid.as_str(), entry.start, entry.end)
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
