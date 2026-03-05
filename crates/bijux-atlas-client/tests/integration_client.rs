use criterion as _;
// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_client::{AtlasClient, ClientConfig, DatasetQuery};
use reqwest as _;
use serde as _;
use serde_json as _;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn spawn_json_server(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0_u8; 1024];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://{}", addr)
}

#[test]
fn dataset_query_parses_response_rows() {
    let body = r#"{"data":{"rows":[{"gene_id":"ENSG000001"}]},"page":{"next_cursor":"abc"}}"#;
    let base_url = spawn_json_server(body);
    let config = ClientConfig {
        base_url,
        ..ClientConfig::default()
    };
    let client = AtlasClient::new(config).expect("client init");
    let query = DatasetQuery::new("110", "homo_sapiens", "GRCh38");
    let page = client.dataset_query(&query, None).expect("query result");
    assert_eq!(page.items.len(), 1);
    assert!(page.next.is_some());
}
