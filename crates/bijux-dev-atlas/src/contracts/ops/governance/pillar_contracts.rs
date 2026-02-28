// SPDX-License-Identifier: Apache-2.0

fn pillar_contracts() -> Vec<Contract> {
    let mut rows = Vec::new();
    rows.extend(datasets_contracts());
    rows.extend(e2e_contracts());
    rows.extend(env_contracts());
    rows.extend(inventory_contracts());
    rows.extend(k8s_contracts());
    rows.extend(load_contracts());
    rows.extend(observe_contracts());
    rows.extend(report_contracts());
    rows.extend(schema_contracts());
    rows.extend(stack_contracts());
    rows
}
