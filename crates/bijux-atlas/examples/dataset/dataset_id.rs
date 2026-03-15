fn main() {
    let dataset = bijux_atlas::model::DatasetId::new("110", "homo_sapiens", "GRCh38")
        .expect("dataset id should be valid");
    println!("dataset={dataset:?}");
}
