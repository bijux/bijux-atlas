// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterSafetyPolicy {
    expected_context: String,
    namespace: String,
}

impl ClusterSafetyPolicy {
    #[must_use]
    pub fn for_kind_profile(kind_profile: &str, namespace: &str) -> Self {
        Self {
            expected_context: expected_kind_context(kind_profile),
            namespace: namespace.to_string(),
        }
    }

    #[must_use]
    pub fn expected_context(&self) -> &str {
        &self.expected_context
    }

    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    #[must_use]
    pub fn allows_context(&self, current_context: &str, force: bool) -> bool {
        is_context_allowed(&self.expected_context, current_context, force)
    }

    #[must_use]
    pub fn context_guard_message(&self, current_context: &str) -> String {
        format!(
            "kubectl context guard failed: expected `{}` got `{}`; pass --force to override",
            self.expected_context, current_context
        )
    }

    #[must_use]
    pub fn namespace_guard_message(&self, detail: &str) -> String {
        format!("namespace guard failed for `{}`: {detail}", self.namespace)
    }
}

#[must_use]
pub fn expected_kind_context(kind_profile: &str) -> String {
    format!("kind-{kind_profile}")
}

#[must_use]
pub fn is_context_allowed(expected: &str, current: &str, force: bool) -> bool {
    current == expected || force
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_profile_maps_to_expected_context() {
        assert_eq!(expected_kind_context("normal"), "kind-normal");
    }

    #[test]
    fn cluster_safety_policy_tracks_expected_context_and_namespace() {
        let policy = ClusterSafetyPolicy::for_kind_profile("normal", "bijux-atlas");
        assert_eq!(policy.expected_context(), "kind-normal");
        assert_eq!(policy.namespace(), "bijux-atlas");
    }

    #[test]
    fn cluster_safety_policy_honors_force_override() {
        let policy = ClusterSafetyPolicy::for_kind_profile("normal", "bijux-atlas");
        assert!(!policy.allows_context("prod-cluster", false));
        assert!(policy.allows_context("prod-cluster", true));
        assert!(policy.allows_context("kind-normal", false));
    }
}
