use criterion as _;
// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_client::{AtlasClient, ClientConfig, DatasetQuery, ErrorClass};
use reqwest as _;
use serde as _;
use serde_json as _;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn spawn_status_server(status: u16, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0_u8; 1024];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 {} TEST\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://{}", addr)
}

#[test]
fn rate_limit_status_maps_to_rate_limited_error() {
    let base_url = spawn_status_server(429, "{}");
    let config = ClientConfig {
        base_url,
        retry_attempts: 1,
        ..ClientConfig::default()
    };
    let client = AtlasClient::new(config).expect("client init");
    let query = DatasetQuery::new("110", "homo_sapiens", "GRCh38");
    let err = client.dataset_query(&query, None).expect_err("must fail");
    assert_eq!(err.class, ErrorClass::RateLimited);
}
