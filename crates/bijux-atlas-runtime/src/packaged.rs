// SPDX-License-Identifier: Apache-2.0

pub(crate) const ENV_CONTRACT_SCHEMA_JSON: &str =
    include_str!("../resources/contracts/env.schema.json");
#[cfg(test)]
pub(crate) const ERROR_CODES_JSON: &str =
    include_str!("../resources/observability/error-codes.json");
pub(crate) const AUTH_POLICY_YAML: &str = include_str!("../resources/security/policy.yaml");
pub(crate) const PERMISSIONS_YAML: &str = include_str!("../resources/security/permissions.yaml");
pub(crate) const ROLES_YAML: &str = include_str!("../resources/security/roles.yaml");

#[cfg(test)]
mod tests {
    use super::{
        AUTH_POLICY_YAML, ENV_CONTRACT_SCHEMA_JSON, ERROR_CODES_JSON, PERMISSIONS_YAML, ROLES_YAML,
    };
    use std::fs;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    fn assert_matches_repo_source(relative_path: &str, packaged: &str) {
        let source_path = workspace_root().join(relative_path);
        if !source_path.exists() {
            return;
        }
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", source_path.display()));
        assert_eq!(
            packaged,
            source,
            "packaged resource drifted from {}",
            source_path.display()
        );
    }

    #[test]
    fn packaged_security_policy_matches_repo_source() {
        assert_matches_repo_source("configs/sources/security/policy.yaml", AUTH_POLICY_YAML);
    }

    #[test]
    fn packaged_permission_catalog_matches_repo_source() {
        assert_matches_repo_source(
            "configs/sources/security/permissions.yaml",
            PERMISSIONS_YAML,
        );
    }

    #[test]
    fn packaged_role_catalog_matches_repo_source() {
        assert_matches_repo_source("configs/sources/security/roles.yaml", ROLES_YAML);
    }

    #[test]
    fn packaged_env_contract_matches_repo_source() {
        assert_matches_repo_source(
            "configs/schemas/contracts/env.schema.json",
            ENV_CONTRACT_SCHEMA_JSON,
        );
    }

    #[test]
    fn packaged_error_codes_match_repo_source() {
        assert_matches_repo_source(
            "configs/sources/operations/observability/error-codes.json",
            ERROR_CODES_JSON,
        );
    }
}
