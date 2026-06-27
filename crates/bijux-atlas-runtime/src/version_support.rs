// SPDX-License-Identifier: Apache-2.0

use semver::Version;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const VERSION_SOURCE_OVERRIDE: &str = "override";
pub const VERSION_SOURCE_GIT_TAG: &str = "git-tag";
pub const VERSION_SOURCE_GIT_TAG_DERIVED: &str = "git-tag-derived";
pub const VERSION_SOURCE_PACKAGE_FALLBACK: &str = "package-fallback";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildVersionMetadata {
    pub semver_version: String,
    pub display_version: String,
    pub source: &'static str,
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeVersionEnv {
    pub name: &'static str,
    pub package_version: &'static str,
    pub build_semver: Option<&'static str>,
    pub build_display: Option<&'static str>,
    pub build_source: Option<&'static str>,
    pub build_git_commit: Option<&'static str>,
    pub build_git_dirty: Option<&'static str>,
    pub build_profile: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeVersionInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub semver: &'static str,
    pub source: &'static str,
    pub git_commit: Option<&'static str>,
    pub git_dirty: Option<bool>,
    pub build_profile: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitDerivedVersion {
    base_semver: String,
    commits_since_tag: u64,
    commit_abbrev: String,
    dirty: bool,
}

pub fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| manifest_dir.to_path_buf(), Path::to_path_buf)
}

pub fn resolve_runtime_versions(
    workspace_root: &Path,
    package_version: &str,
    override_env: &str,
) -> BuildVersionMetadata {
    let git_commit = git_commit_abbrev(workspace_root);
    let git_dirty = git_dirty_state(workspace_root);

    if let Some(override_version) = std::env::var(override_env)
        .ok()
        .and_then(|value| normalize_version_string(&value))
    {
        return BuildVersionMetadata {
            semver_version: override_version.clone(),
            display_version: override_version,
            source: VERSION_SOURCE_OVERRIDE,
            git_commit,
            git_dirty,
        };
    }

    if git_dirty != Some(true) {
        if let Some(version) = describe_exact_tag_version(workspace_root) {
            return BuildVersionMetadata {
                semver_version: version.clone(),
                display_version: tagged_display_version(&version),
                source: VERSION_SOURCE_GIT_TAG,
                git_commit,
                git_dirty,
            };
        }
    }

    if let Some(derived) =
        describe_git_version(workspace_root).or_else(|| latest_tag_baseline_version(workspace_root))
    {
        return BuildVersionMetadata {
            semver_version: compatibility_semver_version(
                package_version,
                derived.commits_since_tag,
                &derived.commit_abbrev,
                derived.dirty,
            ),
            display_version: derived_display_version(
                &derived.base_semver,
                derived.commits_since_tag,
                &derived.commit_abbrev,
                derived.dirty,
            ),
            source: VERSION_SOURCE_GIT_TAG_DERIVED,
            git_commit: Some(derived.commit_abbrev),
            git_dirty: Some(derived.dirty),
        };
    }

    let fallback = fallback_package_version(package_version);
    BuildVersionMetadata {
        semver_version: fallback.clone(),
        display_version: tagged_display_version(&fallback),
        source: VERSION_SOURCE_PACKAGE_FALLBACK,
        git_commit,
        git_dirty,
    }
}

pub fn emit_git_rerun_hints<F>(workspace_root: &Path, mut emit: F)
where
    F: FnMut(String),
{
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-parse", "--git-dir"])
        .output();
    let Ok(output) = output else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if git_dir.is_empty() {
        return;
    }

    let git_dir_path = PathBuf::from(&git_dir);
    let resolved = if git_dir_path.is_absolute() {
        git_dir_path
    } else {
        workspace_root.join(git_dir_path)
    };

    for relative in ["HEAD", "packed-refs", "refs/tags", "refs/heads"] {
        emit(format!(
            "cargo:rerun-if-changed={}",
            resolved.join(relative).display()
        ));
    }
}

pub fn emit_optional_env(key: &str, value: Option<&str>) {
    if let Some(value) = value {
        let _ = writeln!(io::stdout(), "cargo:rustc-env={key}={value}");
    }
}

pub const fn runtime_semver(env: RuntimeVersionEnv) -> &'static str {
    match env.build_semver {
        Some(version) => version,
        None => env.package_version,
    }
}

pub const fn runtime_version(env: RuntimeVersionEnv) -> &'static str {
    match env.build_display {
        Some(version) => version,
        None => runtime_semver(env),
    }
}

pub fn runtime_version_source(env: RuntimeVersionEnv) -> &'static str {
    match env.build_source {
        Some(VERSION_SOURCE_OVERRIDE) => VERSION_SOURCE_OVERRIDE,
        Some(VERSION_SOURCE_GIT_TAG) => VERSION_SOURCE_GIT_TAG,
        Some(VERSION_SOURCE_GIT_TAG_DERIVED) => VERSION_SOURCE_GIT_TAG_DERIVED,
        Some(VERSION_SOURCE_PACKAGE_FALLBACK) => VERSION_SOURCE_PACKAGE_FALLBACK,
        Some(_) | None => VERSION_SOURCE_PACKAGE_FALLBACK,
    }
}

pub fn runtime_git_dirty(env: RuntimeVersionEnv) -> Option<bool> {
    match env.build_git_dirty {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}

pub fn runtime_version_info(env: RuntimeVersionEnv) -> RuntimeVersionInfo {
    RuntimeVersionInfo {
        name: env.name,
        version: runtime_version(env),
        semver: runtime_semver(env),
        source: runtime_version_source(env),
        git_commit: env.build_git_commit,
        git_dirty: runtime_git_dirty(env),
        build_profile: env.build_profile,
    }
}

pub fn runtime_version_line(env: RuntimeVersionEnv) -> String {
    let info = runtime_version_info(env);
    let mut line = format!("{} version {} ({})", info.name, info.version, info.source);
    if info.source != VERSION_SOURCE_GIT_TAG_DERIVED {
        if let Some(commit) = info.git_commit {
            line.push_str(", build ");
            line.push_str(commit);
            if info.git_dirty == Some(true) {
                line.push_str("-dirty");
            }
        }
    }
    line
}

fn describe_exact_tag_version(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["describe", "--tags", "--match", "v[0-9]*", "--exact-match"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let tag = String::from_utf8_lossy(&output.stdout);
    normalize_version_string(tag.trim())
}

fn describe_git_version(workspace_root: &Path) -> Option<GitDerivedVersion> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args([
            "describe",
            "--tags",
            "--match",
            "v[0-9]*",
            "--long",
            "--dirty",
            "--abbrev=12",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_git_describe(String::from_utf8_lossy(&output.stdout).trim())
}

fn parse_git_describe(raw: &str) -> Option<GitDerivedVersion> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (without_dirty, dirty) = match trimmed.strip_suffix("-dirty") {
        Some(value) => (value, true),
        None => (trimmed, false),
    };

    let (tag_and_count, commit_abbrev) = without_dirty.rsplit_once("-g")?;
    if commit_abbrev.trim().is_empty() {
        return None;
    }

    let (tag, count_raw) = tag_and_count.rsplit_once('-')?;
    let commits_since_tag = count_raw.parse::<u64>().ok()?;
    let base_semver = normalize_version_string(tag)?;
    Some(GitDerivedVersion {
        base_semver,
        commits_since_tag,
        commit_abbrev: commit_abbrev.trim().to_string(),
        dirty,
    })
}

fn latest_tag_baseline_version(workspace_root: &Path) -> Option<GitDerivedVersion> {
    let tag = latest_version_tag(workspace_root)?;
    let base_semver = normalize_version_string(&tag)?;
    let commit_abbrev = git_commit_abbrev(workspace_root)?;
    let dirty = git_dirty_state(workspace_root)?;
    let commits_since_tag = commits_since_tag(workspace_root, &tag)?;
    Some(GitDerivedVersion {
        base_semver,
        commits_since_tag,
        commit_abbrev,
        dirty,
    })
}

fn latest_version_tag(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["tag", "--list", "v[0-9]*", "--sort=-version:refname"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn commits_since_tag(workspace_root: &Path, tag: &str) -> Option<u64> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-list", "--count", &format!("{tag}..HEAD")])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .ok()
}

fn tagged_display_version(version: &str) -> String {
    format!("v{version}")
}

fn derived_display_version(
    base_semver: &str,
    commits_since_tag: u64,
    commit_abbrev: &str,
    dirty: bool,
) -> String {
    let mut version = format!(
        "{}+dev.{}.g{}",
        tagged_display_version(base_semver),
        commits_since_tag,
        commit_abbrev
    );
    if dirty {
        version.push_str(".dirty");
    }
    version
}

fn compatibility_semver_version(
    package_version: &str,
    commits_since_tag: u64,
    commit_abbrev: &str,
    dirty: bool,
) -> String {
    let mut version =
        normalize_version_string(package_version).unwrap_or_else(|| package_version.to_string());
    if commits_since_tag == 0 && !dirty {
        return version;
    }

    let separator = if version.contains('+') { "." } else { "+" };
    version.push_str(separator);
    version.push_str("git.");
    version.push_str(&commits_since_tag.to_string());
    version.push_str(".g");
    version.push_str(commit_abbrev);
    if dirty {
        version.push_str(".dirty");
    }
    version
}

fn fallback_package_version(package_version: &str) -> String {
    normalize_version_string(package_version).unwrap_or_else(|| package_version.to_string())
}

fn normalize_version_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let without_prefix = trimmed.strip_prefix('v').unwrap_or(trimmed);
    let parsed = Version::parse(without_prefix).ok()?;
    Some(parsed.to_string())
}

fn git_commit_abbrev(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if commit.is_empty() {
        return None;
    }
    Some(commit)
}

fn git_dirty_state(workspace_root: &Path) -> Option<bool> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}
