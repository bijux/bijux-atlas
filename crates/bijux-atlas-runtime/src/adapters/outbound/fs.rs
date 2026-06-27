// SPDX-License-Identifier: Apache-2.0

use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn write_audit_file_record(
    file_path: &str,
    max_bytes: u64,
    payload: &serde_json::Value,
) -> io::Result<()> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut encoded = serde_json::to_vec(payload)
        .map_err(|err| io::Error::other(format!("encode audit payload failed: {err}")))?;
    encoded.push(b'\n');
    let current_len = fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    if current_len.saturating_add(encoded.len() as u64) > max_bytes {
        let rotated = PathBuf::from(format!("{file_path}.1"));
        if rotated.exists() {
            fs::remove_file(&rotated)?;
        }
        if path.exists() {
            fs::rename(path, &rotated)?;
        }
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(&encoded)?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_audit_file_record;

    fn chrono_like_unix_millis() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis())
    }

    #[test]
    fn audit_file_rotation_happens_at_max_bytes() {
        let root = std::env::temp_dir().join(format!(
            "atlas-audit-rotation-{}",
            chrono_like_unix_millis()
        ));
        std::fs::create_dir_all(&root).expect("create temp dir");
        let log_path = root.join("audit.log");
        let payload = serde_json::json!({
            "event_id": "audit_query_executed",
            "event_name": "query_executed",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 1,
            "sink": "file",
            "action": "dataset.read",
            "resource_kind": "dataset-id",
            "resource_id": "/v1/datasets"
        });
        write_audit_file_record(log_path.to_str().unwrap_or_default(), 64, &payload)
            .expect("first write");
        write_audit_file_record(log_path.to_str().unwrap_or_default(), 64, &payload)
            .expect("second write");
        assert!(log_path.exists());
        assert!(root.join("audit.log.1").exists());
        let _ = std::fs::remove_file(root.join("audit.log.1"));
        let _ = std::fs::remove_file(&log_path);
        let _ = std::fs::remove_dir(&root);
    }
}
