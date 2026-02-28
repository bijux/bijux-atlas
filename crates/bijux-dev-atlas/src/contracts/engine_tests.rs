#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sample_contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
        fn pass_case(_: &RunContext) -> TestResult {
            TestResult::Pass
        }
        Ok(vec![Contract {
            id: ContractId("DOCKER-001".to_string()),
            title: "sample",
            tests: vec![TestCase {
                id: TestId("docker.sample.pass".to_string()),
                title: "sample pass",
                kind: TestKind::Pure,
                run: pass_case,
            }],
        }])
    }

    fn sample_contracts_failing(_repo_root: &Path) -> Result<Vec<Contract>, String> {
        fn fail_case(_: &RunContext) -> TestResult {
            TestResult::Fail(vec![Violation {
                contract_id: "DOCKER-999".to_string(),
                test_id: "docker.sample.fail".to_string(),
                file: Some("docker/images/runtime/Dockerfile".to_string()),
                line: Some(1),
                message: "sample failure".to_string(),
                evidence: Some("latest".to_string()),
            }])
        }
        Ok(vec![Contract {
            id: ContractId("DOCKER-999".to_string()),
            title: "sample fail",
            tests: vec![TestCase {
                id: TestId("docker.sample.fail".to_string()),
                title: "sample fail",
                kind: TestKind::Pure,
                run: fail_case,
            }],
        }])
    }

    #[test]
    fn pretty_output_is_stable() {
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            allow_k8s: false,
            allow_fs_write: false,
            allow_docker_daemon: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", sample_contracts, Path::new("."), &options).expect("run");
        let pretty = to_pretty(&report);
        assert!(pretty.contains("Contracts: docker (mode=static, duration="));
        assert!(pretty.contains("DOCKER-001 sample"));
        assert!(pretty.contains("docker.sample.pass"));
        assert!(pretty.contains("PASS"));
        assert!(pretty.contains("Summary: 1 contracts, 1 tests: 1 pass, 0 fail, 0 skip, 0 error"));
    }

    #[test]
    fn json_serialization_contains_summary_and_tests() {
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            allow_k8s: false,
            allow_fs_write: false,
            allow_docker_daemon: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report =
            run("docker", sample_contracts_failing, Path::new("."), &options).expect("run");
        let payload = to_json(&report);
        assert_eq!(payload["schema_version"], 1);
        assert_eq!(payload["summary"]["contracts"], 1);
        assert_eq!(payload["summary"]["tests"], 1);
        assert_eq!(payload["summary"]["fail"], 1);
        assert!(payload["summary"]["duration_ms"].is_u64());
        assert!(payload["contracts"][0]["duration_ms"].is_u64());
        assert!(payload["contracts"][0]["checks"][0]["duration_ms"].is_u64());
        assert_eq!(
            payload["tests"][0]["violations"][0]["message"],
            "sample failure"
        );
    }

    #[test]
    fn panic_in_test_case_becomes_error_result() {
        fn panic_case(_: &RunContext) -> TestResult {
            panic!("boom");
        }
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            Ok(vec![Contract {
                id: ContractId("DOCKER-998".to_string()),
                title: "panic case",
                tests: vec![TestCase {
                    id: TestId("docker.sample.panic".to_string()),
                    title: "panic case",
                    kind: TestKind::Pure,
                    run: panic_case,
                }],
            }])
        }
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            allow_k8s: false,
            allow_fs_write: false,
            allow_docker_daemon: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", registry, Path::new("."), &options).expect("run");
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.exit_code(), 1);
    }

    #[test]
    fn json_serialization_includes_group_and_nested_checks() {
        let options = RunOptions {
            mode: Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            allow_k8s: false,
            allow_fs_write: false,
            allow_docker_daemon: false,
            skip_missing_tools: false,
            timeout_seconds: 300,
            fail_fast: false,
            contract_filter: None,
            test_filter: None,
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: None,
        };
        let report = run("docker", sample_contracts, Path::new("."), &options).expect("run");
        let payload = to_json(&report);
        assert_eq!(payload["group"].as_str(), Some("docker"));
        assert!(payload["contracts"][0]["checks"].is_array());
        assert_eq!(
            payload["contracts"][0]["contract_id"].as_str(),
            Some("DOCKER-001")
        );
    }

    #[test]
    fn validate_registry_returns_lints_for_duplicate_ids() {
        fn registry(_: &Path) -> Result<Vec<Contract>, String> {
            fn pass_case(_: &RunContext) -> TestResult {
                TestResult::Pass
            }
            Ok(vec![
                Contract {
                    id: ContractId("DOCKER-001".to_string()),
                    title: "first",
                    tests: vec![TestCase {
                        id: TestId("docker.first.pass".to_string()),
                        title: "first pass",
                        kind: TestKind::Pure,
                        run: pass_case,
                    }],
                },
                Contract {
                    id: ContractId("DOCKER-001".to_string()),
                    title: "second",
                    tests: vec![TestCase {
                        id: TestId("docker.second.pass".to_string()),
                        title: "second pass",
                        kind: TestKind::Pure,
                        run: pass_case,
                    }],
                },
            ])
        }
        let contracts = registry(Path::new(".")).expect("registry");
        let err = validate_registry(&[("docker", contracts.as_slice())]).expect_err("lints");
        assert!(err.iter().any(|lint| lint.code == "duplicate-contract-id"));
    }
}
