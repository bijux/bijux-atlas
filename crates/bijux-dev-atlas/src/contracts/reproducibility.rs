// SPDX-License-Identifier: Apache-2.0
//! Reproducibility verification contract model.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReproScenarioKind {
    Crates,
    DockerImage,
    HelmChart,
    DocsSite,
    ReleaseBundle,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReproScenario {
    pub id: String,
    pub kind: ReproScenarioKind,
    pub description: String,
}

pub fn scenario_catalog() -> Vec<ReproScenario> {
    vec![
        ReproScenario {
            id: "rebuild-crates".to_string(),
            kind: ReproScenarioKind::Crates,
            description: "Rebuild crate artifacts and compare checksums.".to_string(),
        },
        ReproScenario {
            id: "rebuild-docker-image".to_string(),
            kind: ReproScenarioKind::DockerImage,
            description: "Rebuild container image and compare digest metadata.".to_string(),
        },
        ReproScenario {
            id: "rebuild-helm-chart".to_string(),
            kind: ReproScenarioKind::HelmChart,
            description: "Rebuild chart package and compare package checksum.".to_string(),
        },
        ReproScenario {
            id: "rebuild-docs-site".to_string(),
            kind: ReproScenarioKind::DocsSite,
            description: "Rebuild docs output and compare stable file hashes.".to_string(),
        },
        ReproScenario {
            id: "rebuild-release-bundle".to_string(),
            kind: ReproScenarioKind::ReleaseBundle,
            description: "Rebuild release bundle and compare manifest hash.".to_string(),
        },
    ]
}
