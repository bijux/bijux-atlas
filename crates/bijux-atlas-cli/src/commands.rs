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
}

#[derive(Subcommand)]
pub(crate) enum DatasetCommand {
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
