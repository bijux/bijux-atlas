use bijux_atlas_model::DatasetId;
use proptest::prelude::*;
use proptest::test_runner::Config;

proptest! {
    #![proptest_config(Config::with_cases(128))]
    #[test]
    fn dataset_id_canonical_format_has_three_segments(
        release in "[0-9]{1,4}",
        species in "[a-z0-9_]{3,20}",
        assembly in "[A-Za-z0-9._]{3,20}"
    ) {
        let parsed = DatasetId::new(&release, &species, &assembly);
        prop_assume!(parsed.is_ok());
        let id = parsed.expect("dataset id");
        let canonical = id.canonical_string();
        let parts: Vec<&str> = canonical.split('/').collect();
        prop_assert_eq!(parts.len(), 3);
        prop_assert_eq!(parts[0], id.release.as_str());
        prop_assert_eq!(parts[1], id.species.as_str());
        prop_assert_eq!(parts[2], id.assembly.as_str());
    }
}
