include!("validate_ops_inventory/loaded_inputs.rs");
include!("validate_ops_inventory/required_input_checks.rs");
include!("validate_ops_inventory/stack_k8s_observe_validations.rs");
include!("validate_ops_inventory/datasets_and_e2e_validations.rs");
include!("validate_ops_inventory/load_and_report_validations.rs");
include!("validate_ops_inventory/surface_and_filesystem_validations.rs");
include!("validate_ops_inventory/pins_and_version_helpers.rs");

pub fn validate_ops_inventory(repo_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    validate_required_ops_inventory_inputs(repo_root, &mut errors);

    let inputs = match load_ops_inventory_validation_inputs(repo_root, &mut errors) {
        Some(v) => v,
        None => return errors,
    };

    validate_stack_k8s_and_observe_manifests(repo_root, &inputs, &mut errors);
    validate_datasets_and_e2e_manifests(repo_root, &inputs, &mut errors);
    validate_load_and_report_manifests(repo_root, &inputs, &mut errors);
    validate_surface_and_filesystem_policies(repo_root, &inputs, &mut errors);

    errors.sort();
    errors.dedup();
    errors
}

