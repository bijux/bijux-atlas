// SPDX-License-Identifier: Apache-2.0

use regex::Regex;
use std::collections::BTreeMap;

pub fn normalize_tool_version_with_regex(raw: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(raw)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMismatchCode {
    MissingBinary,
    VersionMismatch,
}

impl ToolMismatchCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingBinary => "TOOLS_MISSING_BINARY",
            Self::VersionMismatch => "TOOLS_VERSION_MISMATCH",
        }
    }
}

pub fn parse_tool_overrides(values: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    for raw in values {
        let Some((name, path)) = raw.split_once('=') else {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        };
        let name = name.trim();
        let path = path.trim();
        if name.is_empty() || path.is_empty() {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        }
        out.insert(name.to_string(), path.to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tool_version_extracts_captured_semver() {
        let version =
            normalize_tool_version_with_regex("helm version v3.18.4", r"v(\d+\.\d+\.\d+)");
        assert_eq!(version.as_deref(), Some("3.18.4"));
    }

    #[test]
    fn mismatch_codes_stay_stable() {
        assert_eq!(
            ToolMismatchCode::MissingBinary.as_str(),
            "TOOLS_MISSING_BINARY"
        );
        assert_eq!(
            ToolMismatchCode::VersionMismatch.as_str(),
            "TOOLS_VERSION_MISMATCH"
        );
    }

    #[test]
    fn tool_override_parser_rejects_malformed_entries() {
        let error = parse_tool_overrides(&["helm".to_string()]).expect_err("invalid override");
        assert_eq!(error, "invalid --tool override `helm`; expected name=path");
    }

    #[test]
    fn tool_override_parser_preserves_stable_pairs() {
        let overrides = parse_tool_overrides(&[
            "helm=/opt/bin/helm".to_string(),
            "kubectl=/opt/bin/kubectl".to_string(),
        ])
        .expect("parse overrides");
        assert_eq!(overrides["helm"], "/opt/bin/helm");
        assert_eq!(overrides["kubectl"], "/opt/bin/kubectl");
    }
}
