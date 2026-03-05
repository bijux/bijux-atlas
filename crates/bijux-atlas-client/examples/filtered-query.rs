use bijux_atlas_client::{AtlasClient, ClientConfig, DatasetQuery};
use criterion as _;
use reqwest as _;
use serde as _;
use serde_json as _;

fn main() {
    let client = match AtlasClient::new(ClientConfig::default()) {
        Ok(client) => client,
        Err(_err) => return,
    };
    let mut query = DatasetQuery::new("110", "homo_sapiens", "GRCh38");
    query.filter.gene_id = Some("ENSG000001".to_string());
    query.projection.include_biotype = true;
    if let Err(_err) = client.filtered_query(&query) {
        std::process::exit(1);
    }
}
