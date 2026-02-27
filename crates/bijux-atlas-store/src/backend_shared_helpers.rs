// SPDX-License-Identifier: Apache-2.0

pub fn enforce_dataset_immutability(root: &Path, dataset: &DatasetId) -> Result<(), StoreError> {
    let paths = dataset_artifact_paths(root, dataset);
    if paths.manifest.exists() || paths.sqlite.exists() {
        return Err(StoreError::new(
            StoreErrorCode::Conflict,
            "dataset already published and immutable; existing artifacts must not be overwritten",
        ));
    }
    Ok(())
}

fn write_and_sync(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut f = std::fs::File::create(path)
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    f.write_all(bytes)
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    f.sync_all()
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    Ok(())
}

fn sync_dir(dir: &Path) -> Result<(), StoreError> {
    let f = OpenOptions::new()
        .read(true)
        .open(dir)
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    f.sync_all()
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    Ok(())
}
