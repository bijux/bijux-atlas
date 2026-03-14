use bijux_atlas::client::{AtlasClient, ClientConfig, DatasetQuery};
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
    query.limit = 50;
    if let Err(_err) = client.dataset_scan(&query) {
        std::process::exit(1);
    }
}
