// SPDX-License-Identifier: Apache-2.0

use super::domain_support::OpsProfileRegistry;
use crate::ops_support::StackManifestToml;
use crate::*;
use bijux_atlas_ops::inventory::pins_manifest::StackPinsToml;
use bijux_atlas_ops::inventory::tools_manifest::ToolsToml;
use bijux_atlas_ops::load::manifest::LoadToml;

pub(crate) fn resolve_ops_root(
    repo_root: &Path,
    ops_root: Option<PathBuf>,
) -> Result<PathBuf, OpsCommandError> {
    bijux_atlas_ops::workspace::profiles::resolve_ops_root(repo_root, ops_root)
        .map_err(|err| OpsCommandError::Manifest(err.detail()))
}

pub(crate) fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, OpsCommandError> {
    bijux_atlas_ops::workspace::profiles::load_profiles(ops_root)
        .map_err(|err| OpsCommandError::Profile(err.detail()))
}

pub(crate) fn load_profile_registry(
    ops_root: &Path,
) -> Result<OpsProfileRegistry, OpsCommandError> {
    bijux_atlas_ops::workspace::profiles::load_profile_registry(ops_root)
        .map_err(|err| OpsCommandError::Profile(err.detail()))
}

fn load_toolchain_inventory(ops_root: &Path) -> Result<ToolchainInventory, OpsCommandError> {
    bijux_atlas_ops::inventory::toolchain::load_toolchain_inventory_from_ops_root(ops_root).map_err(
        |err| match err {
            bijux_atlas_ops::inventory::toolchain::ToolchainInventoryError::Read { .. } => {
                OpsCommandError::Manifest(err.detail())
            }
            bijux_atlas_ops::inventory::toolchain::ToolchainInventoryError::Parse { .. } => {
                OpsCommandError::Schema(err.detail())
            }
        },
    )
}

pub(crate) fn load_tools_manifest(repo_root: &Path) -> Result<ToolsToml, OpsCommandError> {
    bijux_atlas_ops::inventory::tools_manifest::load_tools_manifest(repo_root).map_err(|err| {
        match err {
            bijux_atlas_ops::inventory::tools_manifest::ToolsManifestError::Read { .. } => {
                OpsCommandError::Manifest(err.detail())
            }
            bijux_atlas_ops::inventory::tools_manifest::ToolsManifestError::Parse { .. } => {
                OpsCommandError::Schema(err.detail())
            }
        }
    })
}

pub(crate) fn load_stack_pins(repo_root: &Path) -> Result<StackPinsToml, OpsCommandError> {
    bijux_atlas_ops::inventory::pins_manifest::load_pins_manifest(repo_root).map_err(
        |err| match err {
            bijux_atlas_ops::inventory::pins_manifest::PinsManifestError::Read { .. } => {
                OpsCommandError::Manifest(err.detail())
            }
            bijux_atlas_ops::inventory::pins_manifest::PinsManifestError::Parse { .. } => {
                OpsCommandError::Schema(err.detail())
            }
        },
    )
}

pub(crate) fn load_stack_manifest(repo_root: &Path) -> Result<StackManifestToml, OpsCommandError> {
    bijux_atlas_ops::stack::manifest::load_stack_manifest(repo_root).map_err(|err| match err {
        bijux_atlas_ops::stack::manifest::StackManifestLoadError::Read { .. } => {
            OpsCommandError::Manifest(err.detail())
        }
        bijux_atlas_ops::stack::manifest::StackManifestLoadError::Parse { .. } => {
            OpsCommandError::Schema(err.detail())
        }
    })
}

pub(crate) fn load_load_manifest(repo_root: &Path) -> Result<LoadToml, OpsCommandError> {
    bijux_atlas_ops::load::manifest::load_load_manifest(repo_root).map_err(|err| match err {
        bijux_atlas_ops::load::manifest::LoadManifestError::Read { .. } => {
            OpsCommandError::Manifest(err.detail())
        }
        bijux_atlas_ops::load::manifest::LoadManifestError::Parse { .. } => {
            OpsCommandError::Schema(err.detail())
        }
    })
}

pub(crate) fn validate_load_manifest(
    repo_root: &Path,
    manifest: &LoadToml,
) -> Result<Vec<String>, OpsCommandError> {
    Ok(bijux_atlas_ops::load::manifest::validate_load_manifest(
        repo_root, manifest,
    ))
}

pub(crate) fn validate_stack_manifest(
    repo_root: &Path,
    manifest: &StackManifestToml,
) -> Result<Vec<String>, OpsCommandError> {
    Ok(bijux_atlas_ops::stack::manifest::validate_stack_manifest(
        repo_root, manifest,
    ))
}

pub(crate) fn resolve_profile(
    requested: Option<String>,
    profiles: &[StackProfile],
) -> Result<StackProfile, OpsCommandError> {
    bijux_atlas_ops::workspace::profiles::resolve_profile(requested, profiles)
        .map_err(|err| OpsCommandError::Profile(err.detail()))
}

pub(crate) fn run_id_or_default(raw: Option<String>) -> Result<RunId, String> {
    raw.map(|v| RunId::parse(&v))
        .transpose()?
        .map_or_else(|| Ok(RunId::from_seed("ops_run")), Ok)
}

pub(crate) fn load_toolchain_inventory_for_ops(
    ops_root: &Path,
) -> Result<ToolchainInventory, OpsCommandError> {
    load_toolchain_inventory(ops_root)
}
