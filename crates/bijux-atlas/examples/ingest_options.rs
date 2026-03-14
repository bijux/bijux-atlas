fn main() {
    let opts = bijux_atlas::ingest::IngestOptions::default();
    println!("timestamp-policy={:?}", opts.timestamp_policy);
}
