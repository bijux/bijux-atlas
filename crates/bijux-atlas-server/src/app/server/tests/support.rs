// SPDX-License-Identifier: Apache-2.0

pub(super) use super::*;
pub(super) use crate::app::server::state::cache_runtime::cache_storage_runtime::{
    dataset_index_path, local_cache_paths,
};
pub(super) use bijux_atlas_core::sha256_hex;
pub(super) use bijux_atlas_model::dataset::{ArtifactChecksums, ManifestStats};
pub(super) use bijux_atlas_model::{ArtifactManifest, DatasetId};
pub(super) use bijux_atlas_runtime::adapters::outbound::store::testing::FakeStore;
pub(super) use rusqlite::Connection;
pub(super) use std::collections::HashSet;
pub(super) use std::path::{Path, PathBuf};
pub(super) use std::sync::Arc;
pub(super) use std::time::Duration;
pub(super) use tempfile::tempdir;

pub(super) struct CreatedDirGuard {
    path: PathBuf,
    existed: bool,
}

impl CreatedDirGuard {
    pub(super) fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            existed: path.exists(),
        }
    }
}

impl Drop for CreatedDirGuard {
    fn drop(&mut self) {
        if !self.existed && self.path.exists() {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

fn fixture_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("x.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
         CREATE TABLE dataset_stats(dimension TEXT NOT NULL, value TEXT NOT NULL, gene_count INTEGER NOT NULL, PRIMARY KEY (dimension, value));
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10,1,10);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('biotype','pc',1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('seqid','chr1',1);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}

pub(super) fn mk_dataset() -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        ArtifactChecksums::new("a".repeat(64), "b".repeat(64), "c".repeat(64), sqlite_sha),
        ManifestStats::new(1, 1, 1),
    );
    (ds, manifest, sqlite)
}

pub(super) fn mk_dataset_for(release: &str) -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new(release, "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        ArtifactChecksums::new("a".repeat(64), "b".repeat(64), "c".repeat(64), sqlite_sha),
        ManifestStats::new(1, 1, 1),
    );
    (ds, manifest, sqlite)
}
