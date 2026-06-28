// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpCheckResponse {
    pub status: u16,
    pub latency_ms: u128,
    pub body: String,
}

pub fn wait_for_local_port(port: u16, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!("timed out waiting for localhost:{port}"))
}

pub fn perform_http_request(local_port: u16, path: &str) -> Result<HttpCheckResponse, String> {
    let started = Instant::now();
    let mut stream = TcpStream::connect(("127.0.0.1", local_port))
        .map_err(|err| format!("connect failed: {err}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| format!("set read timeout failed: {err}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| format!("set write timeout failed: {err}"))?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write failed: {err}"))?;
    let _ = stream.shutdown(Shutdown::Write);
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|err| format!("read failed: {err}"))?;
    let response_text = String::from_utf8_lossy(&response);
    let mut lines = response_text.lines();
    let status_line = lines.next().unwrap_or_default().to_string();
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let body = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|offset| &response[offset + 4..])
        .unwrap_or_default();
    Ok(HttpCheckResponse {
        status: status_code,
        latency_ms: started.elapsed().as_millis(),
        body: String::from_utf8_lossy(body).to_string(),
    })
}

pub fn perform_http_check(
    local_port: u16,
    path: &str,
    body_digest: impl Fn(&str) -> String,
) -> Result<Value, String> {
    let response = perform_http_request(local_port, path)?;
    Ok(serde_json::json!({
        "path": path,
        "status": response.status,
        "latency_ms": response.latency_ms,
        "body_sha256": body_digest(&response.body)
    }))
}

pub trait PortForwardSession {
    fn kill_and_wait(&mut self);
}

pub trait ServicePortForwardRunner {
    type Session: PortForwardSession;

    fn start_service_port_forward(
        &self,
        repo_root: &Path,
        namespace: &str,
        local_port: u16,
        remote_port: u16,
    ) -> Result<Self::Session, String>;
}

struct KubectlPortForwardSession {
    child: std::process::Child,
}

impl PortForwardSession for KubectlPortForwardSession {
    fn kill_and_wait(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct KubectlServicePortForwardRunner;

impl ServicePortForwardRunner for KubectlServicePortForwardRunner {
    type Session = KubectlPortForwardSession;

    fn start_service_port_forward(
        &self,
        repo_root: &Path,
        namespace: &str,
        local_port: u16,
        remote_port: u16,
    ) -> Result<Self::Session, String> {
        let child = std::process::Command::new("kubectl")
            .args([
                "port-forward",
                "-n",
                namespace,
                "--address",
                "127.0.0.1",
                "service/bijux-atlas",
                &format!("{local_port}:{remote_port}"),
            ])
            .current_dir(repo_root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|err| format!("failed to start kubectl port-forward: {err}"))?;
        Ok(KubectlPortForwardSession { child })
    }
}

fn body_sha256(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn run_service_smoke_checks(
    runner: &impl ServicePortForwardRunner,
    repo_root: &Path,
    namespace: &str,
    local_port: u16,
    body_digest: impl Fn(&str) -> String,
) -> Result<Vec<Value>, String> {
    let mut session = runner.start_service_port_forward(repo_root, namespace, local_port, 8080)?;
    let checks = (|| -> Result<Vec<Value>, String> {
        wait_for_local_port(local_port, Duration::from_secs(10))?;
        let mut rows = Vec::new();
        for path in ["/healthz", "/readyz", "/v1/version"] {
            rows.push(perform_http_check(local_port, path, &body_digest)?);
        }
        Ok(rows)
    })();
    session.kill_and_wait();
    checks
}

fn probe_service_http_path(
    runner: &impl ServicePortForwardRunner,
    repo_root: &Path,
    namespace: &str,
    local_port: u16,
    remote_port: u16,
    path: &str,
) -> Result<HttpCheckResponse, String> {
    let mut session =
        runner.start_service_port_forward(repo_root, namespace, local_port, remote_port)?;
    let result = (|| -> Result<HttpCheckResponse, String> {
        wait_for_local_port(local_port, Duration::from_secs(10))?;
        perform_http_request(local_port, path)
    })();
    session.kill_and_wait();
    result
}

pub fn run_kubectl_service_smoke_checks(
    repo_root: &Path,
    namespace: &str,
    local_port: u16,
) -> Result<Vec<Value>, String> {
    run_service_smoke_checks(
        &KubectlServicePortForwardRunner,
        repo_root,
        namespace,
        local_port,
        body_sha256,
    )
}

pub fn probe_kubectl_service_http_path(
    repo_root: &Path,
    namespace: &str,
    local_port: u16,
    remote_port: u16,
    path: &str,
) -> Result<HttpCheckResponse, String> {
    probe_service_http_path(
        &KubectlServicePortForwardRunner,
        repo_root,
        namespace,
        local_port,
        remote_port,
        path,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    fn spawn_single_response_server(
        listener: TcpListener,
        status_line: &'static str,
        body: &'static str,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0_u8; 1024];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write response");
            }
        })
    }

    #[test]
    fn wait_for_local_port_returns_after_listener_binds() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind listener");
        let port = listener.local_addr().expect("listener addr").port();

        let result = wait_for_local_port(port, Duration::from_secs(1));

        assert!(result.is_ok());
    }

    #[test]
    fn perform_http_request_captures_status_and_body() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind listener");
        let port = listener.local_addr().expect("listener addr").port();
        let server = spawn_single_response_server(listener, "200 OK", "{\"ok\":true}");

        let response = perform_http_request(port, "/healthz").expect("http response");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"ok\":true}");
        server.join().expect("join server");
    }

    struct MockSession {
        closed: Arc<AtomicBool>,
    }

    impl PortForwardSession for MockSession {
        fn kill_and_wait(&mut self) {
            self.closed.store(true, Ordering::SeqCst);
        }
    }

    struct MockRunner {
        closed: Arc<AtomicBool>,
    }

    impl ServicePortForwardRunner for MockRunner {
        type Session = MockSession;

        fn start_service_port_forward(
            &self,
            repo_root: &Path,
            namespace: &str,
            local_port: u16,
            remote_port: u16,
        ) -> Result<Self::Session, String> {
            assert!(repo_root.ends_with("repo"));
            assert_eq!(namespace, "atlas-kind");
            assert_eq!(remote_port, 8080);
            let server_closed = self.closed.clone();
            std::thread::spawn(move || {
                let listener = TcpListener::bind(("127.0.0.1", local_port)).expect("bind listener");
                for _ in 0..4 {
                    let (mut stream, _) = listener.accept().expect("accept stream");
                    let mut buffer = [0_u8; 1024];
                    let Ok(bytes_read) = stream.read(&mut buffer) else {
                        continue;
                    };
                    if bytes_read == 0 {
                        continue;
                    }
                    let request = String::from_utf8_lossy(&buffer);
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap_or("/");
                    let body = format!("response:{path}");
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write response");
                }
                while !server_closed.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(10));
                }
            });
            Ok(MockSession {
                closed: self.closed.clone(),
            })
        }
    }

    #[test]
    fn run_service_smoke_checks_uses_port_forward_owner() {
        let root = Path::new("/repo");
        let closed = Arc::new(AtomicBool::new(false));
        let runner = MockRunner {
            closed: closed.clone(),
        };
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind port picker");
        let port = listener.local_addr().expect("picker addr").port();
        drop(listener);

        let rows = run_service_smoke_checks(&runner, root, "atlas-kind", port, |body| {
            format!("sha:{body}")
        })
        .expect("smoke rows");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["path"], "/healthz");
        assert_eq!(rows[1]["path"], "/readyz");
        assert_eq!(rows[2]["path"], "/v1/version");
        assert!(closed.load(Ordering::SeqCst));
    }

    #[test]
    fn probe_service_http_path_uses_port_forward_owner() {
        let root = Path::new("/repo");
        let closed = Arc::new(AtomicBool::new(false));
        let runner = MockRunner {
            closed: closed.clone(),
        };
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind port picker");
        let port = listener.local_addr().expect("picker addr").port();
        drop(listener);

        let response = probe_service_http_path(&runner, root, "atlas-kind", port, 8080, "/metrics")
            .expect("probe response");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, "response:/metrics");
        assert!(closed.load(Ordering::SeqCst));
    }

    #[test]
    fn kubectl_body_sha256_matches_sha256_digest() {
        let digest = body_sha256("atlas");

        assert_eq!(
            digest,
            "7c82602500857aa6ed0cf38c4c3e4ec645bdcaa82c00b9155eb08be100c778a9"
        );
    }
}
