// SPDX-License-Identifier: Apache-2.0

mod context;
mod debug_collection;
mod drill_contracts;
mod helm_release;
mod kind_cluster;
pub mod paths;
pub mod records;
mod scenario_evidence;
mod smoke_checks;
mod stack_cleanup;
mod stack_install;
mod stack_reset;
mod stack_teardown;

pub use self::context::{ensure_owned_simulation_context, SimulationCommandRunner};
pub use self::debug_collection::{
    debug_collect_payload, describe_collect_payload, emit_debug_bundle_report,
    events_collect_payload, logs_collect_payload, resources_snapshot_payload,
};
pub use self::drill_contracts::drill_contract_payload;
pub use self::helm_release::{
    helm_install_payload, helm_uninstall_payload, helm_upgrade_payload, HelmUpgradeRequest,
};
pub use self::kind_cluster::{
    kind_down_payload, kind_preload_payload, kind_status_payload, kind_up_payload,
};
pub use self::scenario_evidence::{
    scenario_evidence_artifacts, write_deterministic_scenario_evidence, ScenarioEvidenceArtifacts,
    ScenarioEvidenceWriteRequest,
};
pub use self::smoke_checks::smoke_command_payload;
pub use self::stack_cleanup::cleanup_stack_state_payload;
pub use self::stack_install::{stack_install_payload, StackInstallRequest};
pub use self::stack_reset::reset_stack_state_payload;
pub use self::stack_teardown::stack_down_payload;
