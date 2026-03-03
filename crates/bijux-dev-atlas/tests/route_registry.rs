// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::registry::{command_routes, validate_command_routes};

#[test]
fn route_registry_stays_sorted_and_valid() {
    let routes = command_routes();
    validate_command_routes(&routes).expect("route registry");

    let mut sorted = routes
        .iter()
        .map(|route| (route.name, route.id))
        .collect::<Vec<_>>();
    let snapshot = sorted.clone();
    sorted.sort();
    assert_eq!(snapshot, sorted, "route registry must stay sorted");
}

#[test]
fn route_registry_covers_primary_domain_routes() {
    let routes = command_routes();
    for route_name in [
        "ops",
        "docs",
        "configs",
        "governance",
        "security",
        "release",
        "perf",
        "docker",
        "contract",
        "checks",
    ] {
        assert!(
            routes.iter().any(|route| route.name == route_name),
            "expected route `{route_name}`"
        );
    }
}
