// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

include!("contracts_registry.inc.rs");
include!("contracts_static_checks.inc.rs");

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "ROOT-001" => {
            "The repo root is a sealed interface: only the declared top-level files and directories are allowed.".to_string()
        }
        "ROOT-002" => "Only the approved root markdown files may exist at the repo root.".to_string(),
        "ROOT-003" => "Legacy script directories are forbidden at the repo root; the canonical control plane is `bijux dev atlas`.".to_string(),
        "ROOT-004" => "Root-level symlinks are forbidden unless they are explicitly allowlisted.".to_string(),
        "ROOT-005" => "The root Dockerfile must follow the declared canonical policy and point at the runtime image definition.".to_string(),
        "ROOT-006" => "The root Makefile must stay a thin delegator and include only `make/public.mk`.".to_string(),
        "ROOT-007" => "The workspace must use a single Cargo.lock at the repo root, with no nested crate lockfiles.".to_string(),
        "ROOT-008" => "The Rust toolchain must be pinned by a concrete semantic version in `rust-toolchain.toml`.".to_string(),
        "ROOT-009" => "The shared Cargo config may not enable implicit network-fetch defaults.".to_string(),
        "ROOT-010" => "The repo root license must stay on the approved SPDX track.".to_string(),
        "ROOT-011" => "SECURITY.md must include a clear private reporting path and disclosure guidance.".to_string(),
        "ROOT-012" => "CONTRIBUTING.md must point contributors to `bijux dev atlas` as the canonical control plane.".to_string(),
        "ROOT-013" => "CHANGELOG.md must include a versioned release header.".to_string(),
        "ROOT-014" => "The root .gitignore may not hide tracked contract outputs.".to_string(),
        "ROOT-015" => "The repo root must not reintroduce duplicate toolchain authority files beside rust-toolchain.toml.".to_string(),
        "ROOT-016" => "The sealed repo root must be described by a committed root-surface.json manifest that matches the actual root surface.".to_string(),
        "ROOT-017" => "The repo root may not contain binary-like artifact files such as archives, executables, or compiled blobs.".to_string(),
        "ROOT-018" => "Committed root-level `.env` files are forbidden; environment state must stay out of the repo root.".to_string(),
        "ROOT-019" => "The repo root directory surface must stay within the approved top-level directory budget.".to_string(),
        "ROOT-020" => "The root-surface.json manifest must describe only single-segment repo root entries, never nested paths.".to_string(),
        "ROOT-021" => "The repo root must keep `.editorconfig` so shared formatting contracts have a single source.".to_string(),
        "ROOT-022" => "Root license metadata must stay unambiguous and avoid conflicting license families.".to_string(),
        "ROOT-023" => "README.md must keep the canonical root entrypoint sections so the root surface remains navigable and stable.".to_string(),
        "ROOT-024" => "Root documentation files may not reference deleted legacy control-plane surfaces such as `scripts/` or `xtask`.".to_string(),
        "ROOT-025" => "Support routing belongs under docs/ or ops/, not as ad hoc root support files.".to_string(),
        "ROOT-026" => "The repo root may not introduce duplicate policy directories outside the canonical configs/ surface.".to_string(),
        "ROOT-027" => "The root surface manifest must declare configs and ops as SSOT roots.".to_string(),
        "ROOT-028" => "The root surface manifest must declare docs as a governed root and docs/ must exist.".to_string(),
        "ROOT-029" => "The repository must not contain nested git repositories under the main working tree.".to_string(),
        "ROOT-030" => "Vendor directories and vendored blobs may not appear at the repo root.".to_string(),
        "ROOT-031" => "Root files must stay beneath the approved size budget so the root surface remains reviewable.".to_string(),
        "ROOT-032" => "Configs and ops must not reintroduce nested rust toolchain authority files.".to_string(),
        "ROOT-033" => "Release process authority belongs under docs/ or ops/, not as ad hoc root release files.".to_string(),
        "ROOT-034" => "Root documentation must use `bijux dev atlas contracts` as the single contracts interface.".to_string(),
        "ROOT-035" => "make/checks.mk must stay a thin compatibility layer over the Rust make contracts runner.".to_string(),
        "ROOT-036" => "make/makefiles/docker.mk must delegate docker governance to the Rust docker contracts runner.".to_string(),
        "ROOT-037" => "Editor backup files and platform noise such as `.DS_Store` are forbidden anywhere in the repo tree.".to_string(),
        "ROOT-038" => "If .gitattributes is present, it must keep an explicit line-ending normalization policy.".to_string(),
        "ROOT-039" => "Workspace member declarations in Cargo.toml must exactly match the crate directories that exist.".to_string(),
        "ROOT-040" => "Workspace crate directories and package names must stay on the canonical bijux-* naming track.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts root`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts root --mode static"
}
