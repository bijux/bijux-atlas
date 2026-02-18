use super::*;

pub(super) fn verify_ingest_inputs(
    gff3: PathBuf,
    fasta: PathBuf,
    fai: PathBuf,
    output_root: PathBuf,
    allow_network_inputs: bool,
    resume: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    let verified = resolve_verify_and_lock_inputs(
        &gff3,
        &fasta,
        &fai,
        &output_root,
        allow_network_inputs,
        resume,
    )?;
    command_output_adapters::emit_ok(
        output_mode,
        json!({
            "command": "atlas ingest verify-inputs",
            "status": "ok",
            "inputs_lockfile": verified.lockfile_path,
            "resolved": {
                "gff3": verified.gff3_path,
                "fasta": verified.fasta_path,
                "fai": verified.fai_path
            }
        }),
    )?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct InputLockfile {
    schema_version: u64,
    created_at_epoch_seconds: u64,
    sources: Vec<InputLockSource>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct InputLockSource {
    kind: String,
    original: String,
    resolved_url: String,
    output_path: String,
    checksum_sha256: String,
    expected_size_bytes: u64,
    original_filename: String,
    mirrors: Vec<String>,
}

#[derive(Debug)]
pub(super) struct VerifiedInputPaths {
    pub(super) gff3_path: PathBuf,
    pub(super) fasta_path: PathBuf,
    pub(super) fai_path: PathBuf,
    pub(super) lockfile_path: PathBuf,
}

pub(super) fn resolve_verify_and_lock_inputs(
    gff3: &std::path::Path,
    fasta: &std::path::Path,
    fai: &std::path::Path,
    output_root: &std::path::Path,
    allow_network_inputs: bool,
    resume: bool,
) -> Result<VerifiedInputPaths, String> {
    let ingest_inputs_dir = output_root.join("_ingest_inputs");
    let tmp_dir = ingest_inputs_dir.join(".tmp");
    let quarantine_dir = ingest_inputs_dir.join("quarantine");
    let lockfile_path = ingest_inputs_dir.join("inputs.lock.json");
    fs::create_dir_all(&ingest_inputs_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&quarantine_dir).map_err(|e| e.to_string())?;

    let specs = [("gff3", gff3), ("fasta", fasta), ("fai", fai)];
    if resume && lockfile_path.exists() {
        let existing: InputLockfile = serde_json::from_slice(
            &fs::read(&lockfile_path).map_err(|e| format!("read lockfile failed: {e}"))?,
        )
        .map_err(|e| format!("parse lockfile failed: {e}"))?;
        for (kind, src) in specs {
            let src_text = src.to_string_lossy().to_string();
            let entry = existing
                .sources
                .iter()
                .find(|x| x.kind == kind)
                .ok_or_else(|| format!("resume lockfile missing entry for {kind}"))?;
            if entry.original != src_text {
                return Err(format!(
                    "resume lockfile mismatch for {kind}: expected original `{}` got `{}`",
                    entry.original, src_text
                ));
            }
            let p = PathBuf::from(&entry.output_path);
            if !p.exists() {
                return Err(format!("resume file missing for {kind}: {}", p.display()));
            }
            let bytes = fs::read(&p).map_err(|e| e.to_string())?;
            let hash = sha256_hex(&bytes);
            if hash != entry.checksum_sha256 || bytes.len() as u64 != entry.expected_size_bytes {
                return Err(format!(
                    "resume lockfile TOCTOU mismatch for {kind}: hash/size changed"
                ));
            }
        }
        return Ok(VerifiedInputPaths {
            gff3_path: PathBuf::from(
                existing
                    .sources
                    .iter()
                    .find(|x| x.kind == "gff3")
                    .ok_or_else(|| "lockfile missing gff3".to_string())?
                    .output_path
                    .clone(),
            ),
            fasta_path: PathBuf::from(
                existing
                    .sources
                    .iter()
                    .find(|x| x.kind == "fasta")
                    .ok_or_else(|| "lockfile missing fasta".to_string())?
                    .output_path
                    .clone(),
            ),
            fai_path: PathBuf::from(
                existing
                    .sources
                    .iter()
                    .find(|x| x.kind == "fai")
                    .ok_or_else(|| "lockfile missing fai".to_string())?
                    .output_path
                    .clone(),
            ),
            lockfile_path,
        });
    }

    let mut lock_sources = Vec::new();
    let mut resolved_paths = std::collections::HashMap::new();
    for (kind, src) in specs {
        let src_text = src.to_string_lossy().to_string();
        let r = resolve_single_input(
            kind,
            &src_text,
            &ingest_inputs_dir,
            &tmp_dir,
            &quarantine_dir,
            allow_network_inputs,
        )?;
        resolved_paths.insert(kind.to_string(), r.0.clone());
        lock_sources.push(r.1);
    }
    let lock = InputLockfile {
        schema_version: 1,
        created_at_epoch_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs(),
        sources: lock_sources,
    };
    let lock_tmp = lockfile_path.with_extension("json.tmp");
    fs::write(
        &lock_tmp,
        canonical::stable_json_bytes(&lock).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&lock_tmp, &lockfile_path).map_err(|e| e.to_string())?;
    Ok(VerifiedInputPaths {
        gff3_path: resolved_paths
            .remove("gff3")
            .ok_or_else(|| "missing resolved gff3".to_string())?,
        fasta_path: resolved_paths
            .remove("fasta")
            .ok_or_else(|| "missing resolved fasta".to_string())?,
        fai_path: resolved_paths
            .remove("fai")
            .ok_or_else(|| "missing resolved fai".to_string())?,
        lockfile_path,
    })
}

fn resolve_single_input(
    kind: &str,
    original: &str,
    ingest_inputs_dir: &std::path::Path,
    tmp_dir: &std::path::Path,
    quarantine_dir: &std::path::Path,
    allow_network_inputs: bool,
) -> Result<(PathBuf, InputLockSource), String> {
    let (resolved_url, source_bytes, original_filename) = if original.starts_with("http://")
        || original.starts_with("https://")
    {
        if !allow_network_inputs {
            return Err(format!(
                "network input forbidden by policy for {kind}; rerun with --allow-network-inputs"
            ));
        }
        let resp = reqwest::blocking::get(original).map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("download failed for {kind}: {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| e.to_string())?.to_vec();
        let filename = original.rsplit('/').next().unwrap_or(kind).to_string();
        (original.to_string(), bytes, filename)
    } else if original.starts_with("s3://") {
        if !allow_network_inputs {
            return Err(format!(
                "network input forbidden by policy for {kind}; rerun with --allow-network-inputs"
            ));
        }
        let endpoint = std::env::var("ATLAS_S3_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string());
        let key = original.trim_start_matches("s3://");
        let url = format!("{}/{}", endpoint.trim_end_matches('/'), key);
        let resp = reqwest::blocking::get(&url).map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("download failed for {kind}: {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| e.to_string())?.to_vec();
        let filename = key.rsplit('/').next().unwrap_or(kind).to_string();
        (url, bytes, filename)
    } else {
        let path = if let Some(p) = original.strip_prefix("file://") {
            PathBuf::from(p)
        } else {
            PathBuf::from(original)
        };
        let bytes = fs::read(&path).map_err(|e| format!("read local input failed: {e}"))?;
        let filename = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or(kind)
            .to_string();
        (path.display().to_string(), bytes, filename)
    };

    let (normalized_bytes, final_name) = match decompress_if_needed(original_filename, &source_bytes)
    {
        Ok(v) => v,
        Err(e) => {
            let q = quarantine_dir.join(format!(
                "{kind}-decode-{}.bad",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|x| x.to_string())?
                    .as_secs()
            ));
            fs::write(&q, &source_bytes).map_err(|x| x.to_string())?;
            return Err(format!(
                "decompression failed for {kind}: {e}; quarantined at {}",
                q.display()
            ));
        }
    };
    let final_path = ingest_inputs_dir.join(format!("{kind}-{final_name}"));
    let tmp_path = tmp_dir.join(format!("{kind}-{final_name}.part"));
    fs::write(&tmp_path, &normalized_bytes).map_err(|e| e.to_string())?;
    let verify_bytes = fs::read(&tmp_path).map_err(|e| e.to_string())?;
    if verify_bytes != normalized_bytes {
        let q = quarantine_dir.join(format!(
            "{kind}-{}.bad",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs()
        ));
        let _ = fs::rename(&tmp_path, &q);
        return Err(format!(
            "download verification failed for {kind}; quarantined at {}",
            q.display()
        ));
    }
    fs::rename(&tmp_path, &final_path).map_err(|e| e.to_string())?;
    let checksum = sha256_hex(&normalized_bytes);
    let source = InputLockSource {
        kind: kind.to_string(),
        original: original.to_string(),
        resolved_url,
        output_path: final_path.display().to_string(),
        checksum_sha256: checksum,
        expected_size_bytes: normalized_bytes.len() as u64,
        original_filename: final_name.clone(),
        mirrors: vec![],
    };
    Ok((final_path, source))
}

fn decompress_if_needed(filename: String, bytes: &[u8]) -> Result<(Vec<u8>, String), String> {
    if filename.ends_with(".gz") {
        let mut decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(bytes));
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut out).map_err(|e| e.to_string())?;
        return Ok((out, filename.trim_end_matches(".gz").to_string()));
    }
    if filename.ends_with(".zst") {
        let mut decoder =
            zstd::stream::read::Decoder::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut out).map_err(|e| e.to_string())?;
        return Ok((out, filename.trim_end_matches(".zst").to_string()));
    }
    Ok((bytes.to_vec(), filename))
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)] // ATLAS-EXC-0002
mod tests {
    use super::resolve_verify_and_lock_inputs;
    use std::fs;

    #[test]
    fn resume_fails_on_lockfile_toc_tou_hash_mismatch() {
        let td = tempfile::tempdir().expect("tmp");
        let root = td.path().join("out");
        let src = td.path().join("genes.gff3");
        let fasta = td.path().join("genome.fa");
        let fai = td.path().join("genome.fa.fai");
        fs::write(&src, "chr1\ts\tgene\t1\t2\t.\t+\t.\tID=g1\n").expect("write gff3");
        fs::write(&fasta, ">chr1\nAC\n").expect("write fasta");
        fs::write(&fai, "chr1\t2\t0\t2\t3\n").expect("write fai");

        let first = resolve_verify_and_lock_inputs(&src, &fasta, &fai, &root, false, false)
            .expect("initial lock");
        fs::write(&first.gff3_path, "tampered").expect("tamper");
        let err =
            resolve_verify_and_lock_inputs(&src, &fasta, &fai, &root, false, true).expect_err(
                "resume must fail when file hash diverges from lockfile (TOCTOU protection)",
            );
        assert!(err.contains("TOCTOU mismatch"));
    }

    #[test]
    fn corrupted_gzip_input_is_quarantined() {
        let td = tempfile::tempdir().expect("tmp");
        let root = td.path().join("out");
        let src = td.path().join("genes.gff3.gz");
        let fasta = td.path().join("genome.fa");
        let fai = td.path().join("genome.fa.fai");
        fs::write(&src, "not-a-gzip").expect("write bad gzip");
        fs::write(&fasta, ">chr1\nAC\n").expect("write fasta");
        fs::write(&fai, "chr1\t2\t0\t2\t3\n").expect("write fai");

        let err = resolve_verify_and_lock_inputs(&src, &fasta, &fai, &root, false, false)
            .expect_err("bad gzip must fail");
        assert!(err.contains("quarantined"));
        let quarantine = root.join("_ingest_inputs").join("quarantine");
        let entries = fs::read_dir(quarantine).expect("read quarantine");
        assert!(entries.count() > 0);
    }

    #[test]
    fn network_inputs_are_forbidden_by_default() {
        let td = tempfile::tempdir().expect("tmp");
        let root = td.path().join("out");
        let err = resolve_verify_and_lock_inputs(
            std::path::Path::new("https://example.invalid/genes.gff3"),
            std::path::Path::new("https://example.invalid/genome.fa"),
            std::path::Path::new("https://example.invalid/genome.fa.fai"),
            &root,
            false,
            false,
        )
        .expect_err("network should be blocked unless explicitly allowed");
        assert!(err.contains("network input forbidden by policy"));
    }
}
