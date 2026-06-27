// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::Region;
use proptest::prelude::*;
use proptest::test_runner::Config;

proptest! {
    #![proptest_config(Config::with_cases(128))]
    #[test]
    fn region_parse_format_roundtrip(
        seqid in "[A-Za-z0-9_\\.]{1,16}",
        start in 1_u64..100_000_u64,
        len in 0_u64..10_000_u64
    ) {
        let end = start + len;
        let raw = format!("{seqid}:{start}-{end}");
        let parsed = Region::parse(&raw).expect("region parse");
        prop_assert_eq!(parsed.canonical_string(), raw);
    }
}
