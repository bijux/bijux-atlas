fn load_json<T: for<'de> Deserialize<'de>>(repo_root: &Path, rel: &str) -> Result<T, String> {
    let path = repo_root.join(rel);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

pub fn load_ops_inventory(repo_root: &Path) -> Result<OpsInventory, String> {
    Ok(OpsInventory {
        stack_profiles: load_json(repo_root, OPS_STACK_PROFILES_PATH)?,
        stack_version_manifest: load_json(repo_root, OPS_STACK_VERSION_MANIFEST_PATH)?,
        toolchain: load_json(repo_root, OPS_TOOLCHAIN_PATH)?,
        surfaces: load_json(repo_root, OPS_SURFACES_PATH)?,
        mirror_policy: load_json(repo_root, OPS_MIRROR_POLICY_PATH)?,
        contracts: load_json(repo_root, OPS_CONTRACTS_PATH)?,
    })
}

fn inventory_fingerprint(repo_root: &Path) -> Result<u64, String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for rel in INVENTORY_INPUTS {
        rel.hash(&mut hasher);
        let path = repo_root.join(rel);
        let bytes =
            fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        bytes.hash(&mut hasher);
    }
    Ok(hasher.finish())
}

pub fn load_ops_inventory_cached(repo_root: &Path) -> Result<OpsInventory, String> {
    let canonical_root = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    let fingerprint = inventory_fingerprint(&canonical_root)?;
    let cache = OPS_INVENTORY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(entry) = cache
        .lock()
        .map_err(|_| "ops inventory cache mutex poisoned".to_string())?
        .get(&canonical_root)
        .cloned()
    {
        if entry.fingerprint == fingerprint {
            return Ok(entry.inventory);
        }
    }
    let inventory = load_ops_inventory(&canonical_root)?;
    cache
        .lock()
        .map_err(|_| "ops inventory cache mutex poisoned".to_string())?
        .insert(
            canonical_root,
            CacheEntry {
                fingerprint,
                inventory: inventory.clone(),
            },
        );
    Ok(inventory)
}

