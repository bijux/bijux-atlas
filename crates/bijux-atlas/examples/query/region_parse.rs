fn main() {
    let region = bijux_atlas::query::Region::parse("chr1:100-200").expect("valid region");
    println!("region={}:{}-{}", region.seqid.as_str(), region.start, region.end);
}
