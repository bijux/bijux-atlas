#[derive(Debug, Clone)]
struct LocalCachePaths {
    cache_root: PathBuf,
    inputs_dir: PathBuf,
    derived_dir: PathBuf,
    fasta: PathBuf,
    fai: PathBuf,
    sqlite: PathBuf,
    manifest: PathBuf,
    release_gene_index: PathBuf,
}

fn local_cache_paths(root: &Path, cache_key: &str) -> LocalCachePaths {
    let cache_root = root.join(cache_key);
    let inputs_dir = cache_root.join("inputs");
    let derived_dir = cache_root.join("derived");
    LocalCachePaths {
        cache_root,
        inputs_dir: inputs_dir.clone(),
        derived_dir: derived_dir.clone(),
        fasta: inputs_dir.join("genome.fa.bgz"),
        fai: inputs_dir.join("genome.fa.bgz.fai"),
        sqlite: derived_dir.join("gene_summary.sqlite"),
        manifest: derived_dir.join("manifest.json"),
        release_gene_index: derived_dir.join("release_gene_index.json"),
    }
}

fn manifest_cache_key(manifest: &ArtifactManifest) -> String {
    let key = manifest.artifact_hash.trim();
    if !key.is_empty() {
        key.to_string()
    } else if !manifest.checksums.sqlite_sha256.trim().is_empty() {
        manifest.checksums.sqlite_sha256.clone()
    } else {
        sha256_hex(manifest.dataset.canonical_string().as_bytes())
    }
}

fn dataset_index_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    root.join(".dataset-index")
        .join(format!("{}.key", dataset.key_string()))
}

fn safe_cache_key(key: &str) -> Result<(), CacheError> {
    if key.is_empty() || key.contains('/') || key.contains("..") {
        return Err(CacheError("invalid cache key".to_string()));
    }
    Ok(())
}

fn write_atomic_file(path: &Path, bytes: &[u8]) -> Result<(), CacheError> {
    let parent = path
        .parent()
        .ok_or_else(|| CacheError("atomic write missing parent".to_string()))?;
    std::fs::create_dir_all(parent).map_err(|e| CacheError(e.to_string()))?;
    let tmp = parent.join(format!(
        ".{}.tmp.{}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("file"),
        std::process::id()
    ));
    {
        let mut f = std::fs::File::create(&tmp).map_err(|e| CacheError(e.to_string()))?;
        use std::io::Write as _;
        f.write_all(bytes).map_err(|e| CacheError(e.to_string()))?;
        f.sync_all().map_err(|e| CacheError(e.to_string()))?;
    }
    std::fs::rename(&tmp, path).map_err(|e| CacheError(e.to_string()))?;
    if let Ok(dir) = std::fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn acquire_artifact_lease(
    lock_path: &Path,
    timeout: Duration,
) -> Result<std::fs::File, CacheError> {
    let started = Instant::now();
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CacheError(e.to_string()))?;
    }
    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(f) => return Ok(f),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if started.elapsed() >= timeout {
                    return Err(CacheError(
                        "timed out waiting for artifact lease".to_string(),
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(CacheError(e.to_string())),
        }
    }
}

fn ensure_secure_dir(path: &Path) -> Result<(), CacheError> {
    std::fs::create_dir_all(path).map_err(|e| CacheError(e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path).map_err(|e| CacheError(e.to_string()))?;
        let mut mode = metadata.permissions().mode();
        if mode & 0o002 != 0 {
            mode &= !0o002;
            match std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    // Kubernetes volume roots can be non-owned by the container user.
                    // Keep startup resilient when chmod cannot be applied.
                }
                Err(e) => return Err(CacheError(e.to_string())),
            }
        }
    }
    Ok(())
}
