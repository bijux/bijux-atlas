use bijux_atlas::client::{AtlasClient, ClientConfig, DatasetQuery};
use criterion as _;

fn main() {
    use reqwest as _;
    use serde as _;
    use serde_json as _;

    let args = std::env::args().collect::<Vec<_>>();
    let gene_id = args.get(1).cloned();
    let base_url =
        std::env::var("ATLAS_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let config = ClientConfig {
        base_url,
        ..ClientConfig::default()
    };
    let client = match AtlasClient::new(config) {
        Ok(client) => client,
        Err(_err) => std::process::exit(2),
    };

    let query = if let Some(gene_id) = gene_id {
        DatasetQuery::new("110", "homo_sapiens", "GRCh38").with_gene_id(gene_id)
    } else {
        DatasetQuery::new("110", "homo_sapiens", "GRCh38")
    };

    match client.dataset_query(&query, None) {
        Ok(_page) => {}
        Err(_err) => std::process::exit(1),
    }
}
