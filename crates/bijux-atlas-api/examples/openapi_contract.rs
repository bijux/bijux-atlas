fn main() {
    let spec = bijux_atlas_api::openapi_v1_spec();
    println!("openapi-bytes={}", spec.to_string().len());
}
