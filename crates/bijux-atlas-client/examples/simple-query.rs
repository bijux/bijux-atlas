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
    let query = DatasetQuery::new("110", "homo_sapiens", "GRCh38");
    if let Err(err) = client.dataset_query(&query, None) {
        eprintln!("query failed: {err}");
    }
}
