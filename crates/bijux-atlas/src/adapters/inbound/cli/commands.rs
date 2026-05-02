// SPDX-License-Identifier: Apache-2.0

use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum ExportFormat {
    Json,
    Jsonl,
    Csv,
}

#[derive(Subcommand)]
pub(crate) enum QueryCommand {
    Run {
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        gene_id: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        name_prefix: Option<String>,
        #[arg(long)]
        biotype: Option<String>,
        #[arg(long)]
        region: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        allow_full_scan: bool,
    },
    Explain {
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        gene_id: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        name_prefix: Option<String>,
        #[arg(long)]
        biotype: Option<String>,
        #[arg(long)]
        region: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        allow_full_scan: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum InspectCommand {
    Dataset {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
    Db {
        #[arg(long)]
        db: PathBuf,
        #[arg(long, default_value_t = 5)]
        sample_rows: usize,
    },
    Provenance {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum ExportCommand {
    Openapi {
        #[arg(long, default_value = "configs/generated/openapi/v1/openapi.json")]
        out: PathBuf,
    },
    Query {
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
        format: ExportFormat,
        #[arg(long)]
        gene_id: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        name_prefix: Option<String>,
        #[arg(long)]
        biotype: Option<String>,
        #[arg(long)]
        region: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        allow_full_scan: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum CatalogCommand {
    Validate {
        path: PathBuf,
    },
    Publish {
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        catalog: PathBuf,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        explain: bool,
    },
    Rollback {
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
    Promote {
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
    LatestAliasUpdate {
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum DatasetCommand {
    Verify {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long, default_value_t = false)]
        deep: bool,
    },
    #[command(hide = true)]
    Validate {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
    Publish {
        #[arg(long)]
        source_root: PathBuf,
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        explain: bool,
    },
    Pack {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long)]
        out: PathBuf,
    },
    VerifyPack {
        #[arg(long)]
        pack: PathBuf,
    },
    EvidenceVerify {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum DiffCommand {
    Build {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        from_release: String,
        #[arg(long)]
        to_release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long, default_value_t = 10000)]
        max_inline_items: usize,
    },
}

#[derive(Subcommand)]
pub(crate) enum GcCommand {
    Plan {
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        catalog: Vec<PathBuf>,
        #[arg(long, default_value = "ops/inventory/gc-pins.json")]
        pins: PathBuf,
    },
    Apply {
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        catalog: Vec<PathBuf>,
        #[arg(long, default_value = "ops/inventory/gc-pins.json")]
        pins: PathBuf,
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },
}
