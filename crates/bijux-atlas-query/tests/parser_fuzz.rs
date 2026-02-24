// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_query::{
    parse_gene_query_request, GeneFields, GeneFilter, GeneQueryRequest, RegionFilter,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn parser_never_panics_under_random_inputs(
        gene_id in proptest::option::of(".*"),
        name in proptest::option::of(".*"),
        name_prefix in proptest::option::of(".*"),
        biotype in proptest::option::of(".*"),
        seqid in ".*",
        start in any::<u64>(),
        end in any::<u64>(),
        use_region in any::<bool>(),
        limit in 0usize..2000usize,
        allow_full_scan in any::<bool>(),
    ) {
        let region = if use_region {
            Some(RegionFilter { seqid, start, end })
        } else {
            None
        };

        let req = GeneQueryRequest {
            fields: GeneFields::default(),
            filter: GeneFilter {
                gene_id,
                name,
                name_prefix,
                biotype,
                region,
            },
            limit,
            cursor: None,
            dataset_key: None,
            allow_full_scan,
        };

        let _ = parse_gene_query_request(&req);
    }
}
