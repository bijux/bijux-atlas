// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_atlas_model::{artifact_paths, ArtifactPaths, DatasetId};

use crate::IngestOptions;

#[derive(Debug, Clone)]
pub struct IngestInputs {
    pub gff3_path: PathBuf,
    pub fasta_path: PathBuf,
    pub fai_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct IngestJob {
    pub inputs: IngestInputs,
    pub output_layout: ArtifactPaths,
    pub options: IngestOptions,
}

impl IngestJob {
    #[must_use]
    pub fn from_options(options: &IngestOptions) -> Self {
        Self {
            inputs: IngestInputs {
                gff3_path: options.gff3_path.clone(),
                fasta_path: options.fasta_path.clone(),
                fai_path: options.fai_path.clone(),
            },
            output_layout: artifact_paths(&options.output_root, &options.dataset),
            options: options.clone(),
        }
    }

    #[must_use]
    pub fn dataset(&self) -> &DatasetId {
        &self.options.dataset
    }
}
