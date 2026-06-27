// SPDX-License-Identifier: Apache-2.0

use regex::Regex;

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
}
