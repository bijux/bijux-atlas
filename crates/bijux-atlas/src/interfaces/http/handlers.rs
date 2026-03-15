// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::redundant_clone)]

include!("handlers_utilities.rs");
include!("handlers_endpoints.rs");

#[cfg(test)]
mod tests {
    use super::readyz_catalog_ready;

    #[test]
    fn readyz_requires_catalog_when_enabled_and_not_cached_only() {
        assert!(!readyz_catalog_ready(true, false, false));
        assert!(readyz_catalog_ready(true, false, true));
    }

    #[test]
    fn readyz_ignores_catalog_when_cached_only_or_not_required() {
        assert!(readyz_catalog_ready(true, true, false));
        assert!(readyz_catalog_ready(false, false, false));
        assert!(readyz_catalog_ready(false, true, false));
    }

    #[test]
    fn readyz_offline_profile_stays_ready_without_catalog() {
        assert!(readyz_catalog_ready(true, true, false));
    }

    #[test]
    fn readyz_baseline_and_perf_profiles_require_catalog_when_enabled() {
        assert!(!readyz_catalog_ready(true, false, false));
        assert!(readyz_catalog_ready(true, false, true));
    }
}
