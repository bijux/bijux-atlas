#![allow(dead_code)]

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
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_URI,
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
