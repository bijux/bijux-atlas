#![allow(dead_code)] // ATLAS-EXC-0001

use rusqlite::{Connection, OpenFlags};

use crate::CacheError;

pub(crate) fn open_readonly(path: &std::path::Path) -> Result<Connection, CacheError> {
    let uri = format!("file:{}?mode=ro&immutable=1", path.display());
    Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn open_readonly_no_mutex(path: &std::path::Path) -> Result<Connection, CacheError> {
    let uri = format!("file:{}?mode=ro&immutable=1", path.display());
    Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn apply_readonly_pragmas(
    conn: &Connection,
    cache_kib: i64,
    mmap_bytes: i64,
) -> Result<(), CacheError> {
    conn.busy_timeout(std::time::Duration::from_millis(200))
        .map_err(|e| CacheError(e.to_string()))?;
    conn.execute_batch(&format!(
        "PRAGMA query_only=ON; PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-{}; PRAGMA mmap_size={};",
        cache_kib, mmap_bytes,
    ))
    .map_err(|e| CacheError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_readonly_uses_immutable_uri_and_rejects_writes() {
        let tmp = tempfile::NamedTempFile::new().expect("tmp");
        let writable = Connection::open(tmp.path()).expect("open writable");
        writable
            .execute_batch("CREATE TABLE t(id INTEGER PRIMARY KEY, v TEXT);")
            .expect("seed schema");
        drop(writable);

        let conn = open_readonly(tmp.path()).expect("open ro");
        apply_readonly_pragmas(&conn, 1024, 1024 * 1024).expect("pragmas");
        let query_only: i64 = conn
            .query_row("PRAGMA query_only", [], |r| r.get(0))
            .expect("query_only");
        assert_eq!(query_only, 1);

        let write_err = conn
            .execute("INSERT INTO t(v) VALUES('x')", [])
            .expect_err("write must fail in readonly mode");
        let msg = write_err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("readonly") || msg.contains("read-only"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn open_readonly_fails_for_missing_sqlite_path() {
        let missing = std::path::Path::new("/tmp/non-existent-bijux-atlas.sqlite");
        let err = open_readonly(missing).expect_err("missing path must fail startup open");
        assert!(!err.to_string().is_empty());
    }
}
