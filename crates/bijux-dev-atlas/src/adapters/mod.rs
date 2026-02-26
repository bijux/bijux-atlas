// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
//! `adapters` contains concrete host-effect implementations (filesystem, process, environment).
//!
//! Boundary: adapters may depend on `std` host APIs and implement `crate::ports`; they must not
//! depend on core check modules or embed policy/core decision logic.

pub use crate::ports::SystemClock as RealClock;
pub use crate::ports::{AdapterError, Capabilities, Fs, FsWrite, Git, Network, ProcessRunner};

mod bundles;
mod fs;
mod process;
mod workspace_root;
mod world;

pub use bundles::{AdaptersBundle, FixedClock, TestBundle};
pub use fs::{
    canonicalize_from_repo_root, discover_repo_root, ensure_write_path_under_artifacts,
    normalize_line_endings, sorted_non_empty_lines, RealFs,
};
pub use process::{run_subprocess_captured, CommandCapture, RealProcessRunner, SubprocessPolicy};
pub type RealExec = RealProcessRunner;
pub type RealWalk = RealFs;
pub use workspace_root::WorkspaceRoot;
pub use world::{DeniedNetwork, DeniedProcessRunner, FakeWorld, RealGit, RealWorld};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterEvent {
    pub adapter: &'static str,
    pub operation: &'static str,
    pub detail: String,
}

pub trait EventLogger {
    fn log(&self, event: AdapterEvent);
}

#[derive(Debug, Default)]
pub struct NoopLogger;

impl EventLogger for NoopLogger {
    fn log(&self, _event: AdapterEvent) {}
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo_root() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("bijux-dev-atlas-adapter-io-{suffix}"));
        fs::create_dir_all(&root).expect("mkdir");
        root
    }

    #[test]
    fn write_guard_allows_only_artifacts_run_root() {
        let repo_root = temp_repo_root();
        let fs_adapter = RealFs;
        let allowed = PathBuf::from("artifacts/atlas-dev/run_one/report.json");
        let denied = PathBuf::from("ops/out.json");

        let ok = fs_adapter.write_text(&repo_root, "run_one", &allowed, "{}");
        assert!(ok.is_ok());

        let fail = fs_adapter.write_text(&repo_root, "run_one", &denied, "{}");
        assert!(matches!(fail, Err(AdapterError::PathViolation { .. })));
    }

    #[test]
    fn denied_process_runner_blocks_execution() {
        let runner = DeniedProcessRunner;
        let err = runner
            .run("echo", &[], Path::new("."))
            .expect_err("must fail");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "subprocess",
                ..
            }
        ));
    }

    #[test]
    fn denied_network_blocks_fetch() {
        let network = DeniedNetwork;
        let err = network
            .get_text("https://example.com")
            .expect_err("must fail");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "network",
                ..
            }
        ));
    }

    #[test]
    fn capabilities_from_cli_flags_maps_expected_effects() {
        let caps = Capabilities::from_cli_flags(true, false, true, false);
        assert!(caps.fs_write);
        assert!(!caps.subprocess);
        assert!(caps.git);
        assert!(!caps.network);
    }

    #[test]
    fn fake_world_reads_stubbed_file() {
        let repo_root = temp_repo_root();
        let file_path = repo_root.join("docs/INDEX.md");
        let fake = FakeWorld::default().with_file(&file_path, "index");
        let text = fake
            .read_text(&repo_root, Path::new("docs/INDEX.md"))
            .expect("read");
        assert_eq!(text, "index");
    }

    #[test]
    fn subprocess_policy_blocks_non_allowlisted_programs() {
        let repo_root = temp_repo_root();
        let policy = SubprocessPolicy::strict_default();
        let err = run_subprocess_captured("python3", &[], &repo_root, &policy).expect_err("deny");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "subprocess",
                ..
            }
        ));
    }

    #[test]
    fn repo_root_discovery_has_explicit_failure_mode() {
        let repo_root = temp_repo_root();
        let nested = repo_root.join("deep/nested");
        fs::create_dir_all(&nested).expect("mkdir nested");
        let err = discover_repo_root(&nested).expect_err("must fail");
        assert!(matches!(err, AdapterError::PathViolation { .. }));
    }

    #[test]
    fn denied_network_is_default_in_real_world_bundle() {
        let world = RealWorld::new();
        let err = world
            .network
            .get_text("https://example.com")
            .expect_err("must deny");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "network",
                ..
            }
        ));
    }

    #[test]
    fn test_bundle_exposes_walk_and_exec_ports() {
        let repo_root = temp_repo_root();
        let docs = repo_root.join("docs/a.md");
        let bundle = TestBundle::new().with_world(FakeWorld::default().with_file(&docs, "hello"));
        let walked = bundle
            .walker()
            .walk_files(&repo_root, Path::new("docs"))
            .expect("walk");
        assert_eq!(walked, vec![docs]);
        let err = bundle
            .exec()
            .run("cargo", &["--version".to_string()], &repo_root)
            .expect_err("not stubbed");
        assert!(matches!(err, AdapterError::Process { .. }));
    }
}
