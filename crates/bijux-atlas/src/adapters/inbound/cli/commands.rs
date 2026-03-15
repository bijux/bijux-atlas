// SPDX-License-Identifier: Apache-2.0

use clap::Subcommand;
use std::path::PathBuf;

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
