/// Determinism policy marker.
///
/// Core canonicalization and hashing logic must not depend on wall-clock time.
#[must_use]
pub const fn determinism_time_policy() -> &'static str {
    "No wall-clock time allowed in deterministic core paths"
}
