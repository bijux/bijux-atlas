use crate::CacheError;
use rusqlite::Connection;
use std::path::Path;
use std::time::Instant;

pub(crate) trait FileSystemAdapter: Send + Sync {
    fn read(&self, path: &Path) -> Result<Vec<u8>, CacheError>;
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), CacheError>;
    fn create_dir_all(&self, path: &Path) -> Result<(), CacheError>;
    fn rename(&self, src: &Path, dst: &Path) -> Result<(), CacheError>;
}

pub(crate) trait ClockAdapter: Send + Sync {
    fn now(&self) -> Instant;
    fn add_ms(&self, base: Instant, ms: u64) -> Instant;
}

pub(crate) trait RandomAdapter: Send + Sync {
    fn next_u64(&self) -> u64;
}

pub(crate) trait SqliteAdapter: Send + Sync {
    fn open_readonly(&self, path: &Path) -> Result<Connection, CacheError>;
    fn open_readonly_no_mutex(&self, path: &Path) -> Result<Connection, CacheError>;
    fn apply_readonly_pragmas(
        &self,
        conn: &Connection,
        cache_kib: i64,
        mmap_bytes: i64,
    ) -> Result<(), CacheError>;
}

#[derive(Debug, Default)]
pub(crate) struct SystemEffects;

impl FileSystemAdapter for SystemEffects {
    fn read(&self, path: &Path) -> Result<Vec<u8>, CacheError> {
        crate::effect_adapters::fs_adapters::read(path)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), CacheError> {
        crate::effect_adapters::fs_adapters::write(path, bytes)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), CacheError> {
        crate::effect_adapters::fs_adapters::create_dir_all(path)
    }

    fn rename(&self, src: &Path, dst: &Path) -> Result<(), CacheError> {
        crate::effect_adapters::fs_adapters::rename(src, dst)
    }
}

impl ClockAdapter for SystemEffects {
    fn now(&self) -> Instant {
        crate::effect_adapters::clock_adapters::now()
    }

    fn add_ms(&self, base: Instant, ms: u64) -> Instant {
        crate::effect_adapters::clock_adapters::add_ms(base, ms)
    }
}

impl RandomAdapter for SystemEffects {
    fn next_u64(&self) -> u64 {
        crate::effect_adapters::random_adapters::next_u64()
    }
}

impl SqliteAdapter for SystemEffects {
    fn open_readonly(&self, path: &Path) -> Result<Connection, CacheError> {
        crate::effect_adapters::sqlite_adapters::open_readonly(path)
    }

    fn open_readonly_no_mutex(&self, path: &Path) -> Result<Connection, CacheError> {
        crate::effect_adapters::sqlite_adapters::open_readonly_no_mutex(path)
    }

    fn apply_readonly_pragmas(
        &self,
        conn: &Connection,
        cache_kib: i64,
        mmap_bytes: i64,
    ) -> Result<(), CacheError> {
        crate::effect_adapters::sqlite_adapters::apply_readonly_pragmas(conn, cache_kib, mmap_bytes)
    }
}
