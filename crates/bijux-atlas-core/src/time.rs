#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterminismOnly;

impl DeterminismOnly {
    #[must_use]
    pub const fn policy() -> &'static str {
        "No wall-clock time allowed in deterministic core paths"
    }
}

/// Determinism policy marker.
///
/// Core canonicalization and hashing logic must not depend on wall-clock time.
#[must_use]
pub const fn determinism_time_policy() -> &'static str {
    DeterminismOnly::policy()
}
