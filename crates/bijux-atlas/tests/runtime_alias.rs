// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::sha256_hex as alias_sha256_hex;
use bijux_atlas::query::Region as AliasRegion;
use bijux_atlas_runtime::domain::sha256_hex as runtime_sha256_hex;
use bijux_atlas_runtime::query::Region as RuntimeRegion;

#[test]
fn alias_reexports_runtime_root_functions() {
    assert_eq!(
        bijux_atlas::no_randomness_policy(),
        bijux_atlas_runtime::no_randomness_policy()
    );
}

#[test]
fn alias_reexports_runtime_module_functions() {
    assert_eq!(alias_sha256_hex(b"atlas"), runtime_sha256_hex(b"atlas"));
}

#[test]
fn alias_types_match_runtime_types() {
    let runtime_region = RuntimeRegion::parse("chr1:10-20").expect("runtime region");
    let alias_region: AliasRegion = runtime_region;
    assert_eq!(
        alias_region,
        RuntimeRegion::parse("chr1:10-20").expect("runtime region again")
    );
}
