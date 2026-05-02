// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn api_dto_module_does_not_depend_on_domain_modules() {
    let path = crate_root().join("src/contracts/api/dto.rs");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    for forbidden in ["crate::domain::", "bijux_atlas::domain::"] {
        assert!(
            !text.contains(forbidden),
            "api dto module must stay wire-owned and domain-independent"
        );
    }
}

#[test]
fn http_dto_module_uses_wire_categories_not_domain_categories() {
    let path = crate_root().join("src/adapters/inbound/http/dto.rs");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    assert!(
        text.contains("enum FailureInjectionCategory"),
        "http dto module must define wire category enum"
    );
    assert!(
        !text.contains("use crate::domain::cluster::resilience::FailureCategory"),
        "http dto module must not import domain failure category"
    );
}
