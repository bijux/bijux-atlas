use bijux_atlas_client::{AtlasClient, ClientConfig, DatasetQuery};
use reqwest as _;
use serde as _;
use serde_json as _;

fn main() {
    let client = match AtlasClient::new(ClientConfig::default()) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("failed to initialize client: {err}");
            return;
        }
    };
    let mut query = DatasetQuery::new("110", "homo_sapiens", "GRCh38");
    query.limit = 50;
    if let Err(err) = client.dataset_scan(&query) {
        eprintln!("scan failed: {err}");
    }
}
