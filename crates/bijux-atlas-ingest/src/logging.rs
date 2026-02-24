// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestStage {
    Prepare,
    Decode,
    Extract,
    Persist,
    Finalize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestEvent {
    pub stage: IngestStage,
    pub name: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct IngestLog {
    events: Vec<IngestEvent>,
}

impl IngestLog {
    pub fn emit(
        &mut self,
        stage: IngestStage,
        name: impl Into<String>,
        fields: BTreeMap<String, String>,
    ) {
        self.events.push(IngestEvent {
            stage,
            name: name.into(),
            fields,
        });
    }

    #[must_use]
    pub fn events(&self) -> &[IngestEvent] {
        &self.events
    }
}
