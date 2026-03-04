fn main() {
    let dataset = bijux_atlas_core::DatasetId::new("110/homo_sapiens/GRCh38")
        .expect("dataset id should be valid");
    println!("dataset={} policy={}", dataset, bijux_atlas_core::no_randomness_policy());
}
