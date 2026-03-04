// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub log_json: bool,
    pub level: String,
    pub filter_targets: Option<String>,
    pub sampling_rate: f64,
    pub redaction_enabled: bool,
    pub rotation_max_bytes: u64,
    pub rotation_max_files: u32,
}

impl LoggingConfig {
    #[must_use]
    pub fn default_filter_directive(&self) -> String {
        let base = self.level.to_lowercase();
        let targets = self
            .filter_targets
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        match targets {
            Some(targets) => format!("{base},{targets}"),
            None => base,
        }
    }

    #[must_use]
    pub fn should_emit_sampled(&self, stable_key: &str) -> bool {
        if self.sampling_rate >= 1.0 {
            return true;
        }
        if self.sampling_rate <= 0.0 {
            return false;
        }
        let digest = bijux_atlas_core::sha256_hex(stable_key.as_bytes());
        let bucket = u64::from_str_radix(&digest[..8], 16).unwrap_or(0);
        let ratio = (bucket as f64) / (u32::MAX as f64);
        ratio <= self.sampling_rate
    }
}

#[must_use]
pub fn redact_if_needed(redaction_enabled: bool, value: &str) -> String {
    if !redaction_enabled {
        return value.to_string();
    }
    let lower = value.to_ascii_lowercase();
    if lower.contains("secret") || lower.contains("token") || lower.contains("password") {
        "[redacted]".to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_includes_targets_when_present() {
        let cfg = LoggingConfig {
            log_json: true,
            level: "INFO".to_string(),
            filter_targets: Some("atlas=debug,hyper=warn".to_string()),
            sampling_rate: 1.0,
            redaction_enabled: true,
            rotation_max_bytes: 10_485_760,
            rotation_max_files: 5,
        };
        assert_eq!(
            cfg.default_filter_directive(),
            "info,atlas=debug,hyper=warn"
        );
    }

    #[test]
    fn sampled_decision_is_deterministic() {
        let cfg = LoggingConfig {
            log_json: true,
            level: "info".to_string(),
            filter_targets: None,
            sampling_rate: 0.5,
            redaction_enabled: true,
            rotation_max_bytes: 10_485_760,
            rotation_max_files: 5,
        };
        let a = cfg.should_emit_sampled("req-123");
        let b = cfg.should_emit_sampled("req-123");
        assert_eq!(a, b);
    }

    #[test]
    fn redaction_masks_secret_like_values() {
        assert_eq!(redact_if_needed(true, "api_token=abc"), "[redacted]");
        assert_eq!(redact_if_needed(true, "safe-value"), "safe-value");
        assert_eq!(redact_if_needed(false, "api_token=abc"), "api_token=abc");
    }
}
