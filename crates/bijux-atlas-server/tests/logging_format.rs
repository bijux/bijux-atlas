use std::io;
use std::sync::{Arc, Mutex};

use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl<'a> MakeWriter<'a> for SharedBuffer {
    type Writer = BufferWriter;

    fn make_writer(&'a self) -> Self::Writer {
        BufferWriter(Arc::clone(&self.0))
    }
}

impl io::Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self
            .0
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "lock poisoned"))?;
        guard.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn structured_logging_format_is_valid_json() {
    let sink = SharedBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(sink.clone())
        .json()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(
            target: "atlas_audit",
            request_id = "req-123",
            method = "GET",
            path = "/v1/genes",
            status = 200_u16,
            "audit"
        );
    });

    let bytes = sink.0.lock().expect("lock output").clone();
    let text = String::from_utf8(bytes).expect("utf8 log output");
    let line = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .expect("log line");
    let parsed: serde_json::Value = serde_json::from_str(line).expect("json log line");

    assert_eq!(parsed.get("level").and_then(|v| v.as_str()), Some("INFO"));
    assert_eq!(
        parsed.get("target").and_then(|v| v.as_str()),
        Some("atlas_audit")
    );
    let fields = parsed.get("fields").expect("fields object");
    assert_eq!(
        fields.get("request_id").and_then(|v| v.as_str()),
        Some("req-123")
    );
    assert_eq!(
        fields.get("path").and_then(|v| v.as_str()),
        Some("/v1/genes")
    );
}
