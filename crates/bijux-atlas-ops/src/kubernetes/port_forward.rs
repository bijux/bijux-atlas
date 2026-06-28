// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn port_forward_command(resource: &str, local_port: u16, remote_port: u16) -> Vec<String> {
    vec![
        "kubectl".to_string(),
        "port-forward".to_string(),
        "--address".to_string(),
        "127.0.0.1".to_string(),
        resource.to_string(),
        format!("{local_port}:{remote_port}"),
    ]
}

pub fn port_forward_payload(resource: &str, local_port: u16, remote_port: u16) -> Value {
    json!({
        "schema_version": 1,
        "text": "k8s port-forward command prepared",
        "rows": [{
            "resource": resource,
            "local_port": local_port,
            "remote_port": remote_port,
            "argv": port_forward_command(resource, local_port, remote_port)
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_forward_command_binds_to_loopback() {
        let argv = port_forward_command("service/atlas-api", 8080, 80);
        assert_eq!(argv[0], "kubectl");
        assert_eq!(argv[1], "port-forward");
        assert!(argv.contains(&"--address".to_string()));
        assert!(argv.contains(&"127.0.0.1".to_string()));
        assert_eq!(argv.last().expect("forward target should exist"), "8080:80");
    }

    #[test]
    fn port_forward_payload_embeds_the_generated_command() {
        let payload = port_forward_payload("service/atlas-api", 8080, 80);
        assert_eq!(payload["text"], "k8s port-forward command prepared");
        assert_eq!(payload["rows"][0]["resource"], "service/atlas-api");
        assert_eq!(payload["rows"][0]["argv"][5], "8080:80");
    }
}
