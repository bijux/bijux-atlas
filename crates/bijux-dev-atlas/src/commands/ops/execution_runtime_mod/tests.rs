// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn scanner_detects_timestamp_markers() {
        let errors = scan_timestamps("metadata:\n  creationTimestamp: now\n");
        assert!(errors.iter().any(|e| e.contains("creationTimestamp")));
    }

    #[test]
    fn scanner_detects_unpinned_images() {
        let errors = scan_unpinned_images("image: registry.example/app:v1\n");
        assert!(errors.iter().any(|e| e.contains("digest pinned")));
    }

    #[test]
    fn scanner_detects_forbidden_kind() {
        let errors = scan_forbidden_kinds("kind: ClusterRole\n");
        assert!(errors.iter().any(|e| e.contains("ClusterRole")));
    }

    #[test]
    fn context_guard_refuses_unexpected_context_without_force() {
        assert!(!is_context_allowed("kind-normal", "prod-cluster", false));
        assert!(is_context_allowed("kind-normal", "prod-cluster", true));
        assert!(is_context_allowed("kind-normal", "kind-normal", false));
    }

    #[test]
    fn conformance_aggregation_flags_unready_resources() {
        let deployments = serde_json::json!({
            "items":[{"metadata":{"name":"atlas"},"status":{"replicas":2,"readyReplicas":1}}]
        });
        let pods = serde_json::json!({
            "items":[{"metadata":{"name":"atlas-1"},"status":{"phase":"Pending"}}]
        });
        let (errors, rows) = conformance_summary(&deployments, &pods);
        assert_eq!(rows.len(), 2);
        assert!(errors.iter().any(|e| e.contains("deployment")));
        assert!(errors.iter().any(|e| e.contains("pod")));
    }

    #[test]
    fn load_report_parses_k6_summary_and_enforces_thresholds() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/load/thresholds")).expect("mkdir thresholds");
        std::fs::create_dir_all(root.path().join("ops/load/k6/suites")).expect("mkdir suites");
        std::fs::create_dir_all(root.path().join("ops/load/queries")).expect("mkdir queries");
        std::fs::create_dir_all(root.path().join("ops/atlas-dev")).expect("mkdir atlas-dev");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        std::fs::write(
            root.path().join("ops/inventory/registry.toml"),
            "schema_version = 1\n",
        )
        .expect("registry");
        std::fs::write(root.path().join("ops/inventory/tools.toml"), "schema_version = 1\n")
            .expect("tools");
        std::fs::create_dir_all(root.path().join("artifacts/ops/ops_run/load/mixed"))
            .expect("mkdir artifacts");
        std::fs::write(
            root.path().join("ops/load/load.toml"),
            "[suites.mixed]\nscript=\"ops/load/k6/suites/mixed-80-20.js\"\ndataset=\"ops/load/queries/pinned-v1.json\"\nthresholds=\"ops/load/thresholds/mixed.thresholds.json\"\n[suites.mixed.env]\nATLAS_BASE_URL=\"http://127.0.0.1:8080\"\n",
        )
        .expect("manifest");
        std::fs::write(
            root.path()
                .join("ops/load/thresholds/mixed.thresholds.json"),
            "{\"p95_ms_max\":900,\"p99_ms_max\":1200,\"error_rate_max\":0.01}",
        )
        .expect("thresholds");
        std::fs::write(root.path().join("ops/load/k6/suites/mixed-80-20.js"), "").expect("script");
        std::fs::write(root.path().join("ops/load/queries/pinned-v1.json"), "{}").expect("dataset");
        std::fs::write(
            root.path().join("artifacts/ops/ops_run/load/mixed/k6-summary.json"),
            "{\"metrics\":{\"http_req_duration\":{\"values\":{\"p(95)\":1200,\"p(99)\":1500}},\"http_req_failed\":{\"values\":{\"rate\":0.02}}}}",
        )
        .expect("summary");
        let common = crate::cli::OpsCommonArgs {
            repo_root: Some(root.path().to_path_buf()),
            ops_root: None,
            artifacts_root: None,
            profile: None,
            format: crate::cli::FormatArg::Json,
            out: None,
            run_id: Some("ops_run".to_string()),
            strict: false,
            fail_fast: false,
            max_failures: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            force: false,
            tool_overrides: Vec::new(),
        };
        let (rendered, code) = run_ops_load_report(&common, "mixed", None).expect("report");
        assert_eq!(code, 1);
        let payload: Value = serde_json::from_str(&rendered).expect("json");
        assert!(payload["rows"][0]["report"]["violations"]
            .as_array()
            .is_some_and(|v| !v.is_empty()));
    }

    #[test]
    fn load_plan_emits_sorted_env_rows() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/load/k6/suites")).expect("mkdir suites");
        std::fs::create_dir_all(root.path().join("ops/load/queries")).expect("mkdir queries");
        std::fs::create_dir_all(root.path().join("ops/load/thresholds")).expect("mkdir thresholds");
        std::fs::create_dir_all(root.path().join("ops/atlas-dev")).expect("mkdir atlas-dev");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        std::fs::write(
            root.path().join("ops/inventory/registry.toml"),
            "schema_version = 1\n",
        )
        .expect("registry");
        std::fs::write(root.path().join("ops/inventory/tools.toml"), "schema_version = 1\n")
            .expect("tools");
        std::fs::write(
            root.path().join("ops/load/load.toml"),
            "[suites.mixed]\nscript=\"ops/load/k6/suites/mixed-80-20.js\"\ndataset=\"ops/load/queries/pinned-v1.json\"\nthresholds=\"ops/load/thresholds/mixed.thresholds.json\"\n[suites.mixed.env]\nZZZ=\"1\"\nAAA=\"2\"\n",
        )
        .expect("manifest");
        std::fs::write(root.path().join("ops/load/k6/suites/mixed-80-20.js"), "").expect("script");
        std::fs::write(root.path().join("ops/load/queries/pinned-v1.json"), "{}").expect("dataset");
        std::fs::write(
            root.path()
                .join("ops/load/thresholds/mixed.thresholds.json"),
            "{}",
        )
        .expect("thresholds");
        let common = crate::cli::OpsCommonArgs {
            repo_root: Some(root.path().to_path_buf()),
            ops_root: None,
            artifacts_root: None,
            profile: None,
            format: crate::cli::FormatArg::Json,
            out: None,
            run_id: None,
            strict: false,
            fail_fast: false,
            max_failures: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            force: false,
            tool_overrides: Vec::new(),
        };
        let (rendered, code) = run_ops_load_plan(&common, "mixed").expect("plan");
        assert_eq!(code, 0);
        let payload: Value = serde_json::from_str(&rendered).expect("json");
        let env = payload["rows"][0]["env"].as_array().expect("env");
        assert_eq!(env[0]["name"].as_str(), Some("AAA"));
        assert_eq!(env[1]["name"].as_str(), Some("ZZZ"));
    }
}
