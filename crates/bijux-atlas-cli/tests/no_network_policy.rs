// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn cli_unit_test_sources_do_not_embed_network_client_usage() {
    let src = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        .expect("read cli src");
    for forbidden in ["TcpStream::connect", "UdpSocket::bind", "hyper::", "surf::"] {
        assert!(
            !src.contains(forbidden),
            "forbidden network token found in cli source: {forbidden}"
        );
    }
}
