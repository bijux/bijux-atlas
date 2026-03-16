use bijux_atlas::adapters::inbound::client::{AtlasClient, ClientConfig, DatasetQuery};

fn main() {
    let base_url =
        std::env::var("ATLAS_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let release = std::env::var("ATLAS_RELEASE").unwrap_or_else(|_| "110".to_string());
    let species = std::env::var("ATLAS_SPECIES").unwrap_or_else(|_| "homo_sapiens".to_string());
    let assembly = std::env::var("ATLAS_ASSEMBLY").unwrap_or_else(|_| "GRCh38".to_string());

    let mut args = std::env::args().skip(1);
    let gene_id = args.next();

    let client = match AtlasClient::new(ClientConfig {
        base_url: base_url.clone(),
        ..ClientConfig::default()
    }) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("failed to build client for {base_url}: {err}");
            std::process::exit(2);
        }
    };

    let mut query = DatasetQuery::new(release, species, assembly)
        .with_limit(10)
        .include_coords()
        .include_biotype();
    if let Some(gene_id) = gene_id {
        query = query.with_gene_id(gene_id);
    }

    match client.dataset_query(&query, None) {
        Ok(page) => {
            println!("rows={}", page.items.len());
            match page.next {
                Some(cursor) => println!("next_cursor={}", cursor.0),
                None => println!("next_cursor="),
            }
        }
        Err(err) => {
            eprintln!("dataset query failed: {err}");
            std::process::exit(1);
        }
    }
}
