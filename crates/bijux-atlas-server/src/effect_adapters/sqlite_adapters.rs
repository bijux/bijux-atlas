#![allow(dead_code)]

use rusqlite::{Connection, OpenFlags};

use crate::CacheError;

pub(crate) fn open_readonly(path: &std::path::Path) -> Result<Connection, CacheError> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn apply_readonly_pragmas(
    conn: &Connection,
    cache_kib: i64,
    mmap_bytes: i64,
) -> Result<(), CacheError> {
    conn.execute_batch(&format!(
        "PRAGMA query_only=ON; PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-{}; PRAGMA mmap_size={};",
        cache_kib, mmap_bytes,
    ))
    .map_err(|e| CacheError(e.to_string()))
}
