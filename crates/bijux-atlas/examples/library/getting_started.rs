fn main() -> Result<(), String> {
    let dataset = bijux_atlas::model::DatasetId::new("110", "homo_sapiens", "GRCh38")
        .map_err(|err| err.to_string())?;
    let region =
        bijux_atlas::query::Region::parse("chr1:1000-1250").map_err(|err| err.to_string())?;

    println!("dataset={}", dataset.canonical_string());
    println!("region={}", region.canonical_string());
    println!("policy={}", bijux_atlas::no_randomness_policy());

    Ok(())
}
