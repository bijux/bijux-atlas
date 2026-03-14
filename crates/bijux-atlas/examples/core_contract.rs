fn main() {
    let dataset = bijux_atlas::types::DatasetId::new("110/homo_sapiens/GRCh38")
        .expect("dataset id should be valid");
    println!(
        "dataset={} policy={}",
        dataset,
        bijux_atlas::no_randomness_policy()
    );
}
