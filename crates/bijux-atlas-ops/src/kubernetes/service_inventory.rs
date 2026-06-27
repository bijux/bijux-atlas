// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

pub fn service_port_rows(services: &Value) -> Vec<Value> {
    services
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|service| {
            let name = service
                .get("metadata")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let cluster_ip = service
                .get("spec")
                .and_then(|value| value.get("clusterIP"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let ports = service
                .get("spec")
                .and_then(|value| value.get("ports"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "kind":"service_port_discovery",
                "service": name,
                "cluster_ip": cluster_ip,
                "ports": ports
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_inventory_reports_cluster_ip_and_ports() {
        let services = serde_json::json!({
            "items": [{
                "metadata": { "name": "atlas-api" },
                "spec": {
                    "clusterIP": "10.0.0.15",
                    "ports": [{ "name": "http", "port": 8080 }]
                }
            }]
        });

        let rows = service_port_rows(&services);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["kind"], "service_port_discovery");
        assert_eq!(rows[0]["service"], "atlas-api");
        assert_eq!(rows[0]["cluster_ip"], "10.0.0.15");
        assert_eq!(rows[0]["ports"][0]["port"], 8080);
    }
}
