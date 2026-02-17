use bijux_atlas_core::sha256_hex;

#[must_use]
pub fn consistent_route_dataset(dataset_key: &str, nodes: &[String]) -> Option<String> {
    if nodes.is_empty() {
        return None;
    }
    let mut sorted = nodes.to_vec();
    sorted.sort();
    let mut best: Option<(String, String)> = None;
    for node in sorted {
        let score = sha256_hex(format!("{dataset_key}|{node}").as_bytes());
        match &best {
            Some((best_score, _)) if score <= *best_score => {}
            _ => best = Some((score, node)),
        }
    }
    best.map(|(_, n)| n)
}

#[cfg(test)]
mod tests {
    use super::consistent_route_dataset;

    #[test]
    fn routing_is_stable_for_same_inputs() {
        let nodes = vec![
            "pod-b".to_string(),
            "pod-a".to_string(),
            "pod-c".to_string(),
        ];
        let r1 = consistent_route_dataset("110/homo_sapiens/GRCh38", &nodes).expect("route");
        let r2 = consistent_route_dataset("110/homo_sapiens/GRCh38", &nodes).expect("route");
        assert_eq!(r1, r2);
    }

    #[test]
    fn routing_returns_none_for_empty_nodes() {
        assert!(consistent_route_dataset("x", &[]).is_none());
    }
}
