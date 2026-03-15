// SPDX-License-Identifier: Apache-2.0

use super::commands::{CatalogCommand, DatasetCommand, DiffCommand, GcCommand};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf as CliPathBuf;

#[derive(Parser)]
#[command(name = "bijux-atlas")]
#[command(about = "Bijux Atlas operations CLI")]
#[command(help_template = super::BIJUX_HELP_TEMPLATE)]
#[command(
    after_help = "Environment:\n  BIJUX_LOG_LEVEL   Log verbosity override\n  BIJUX_CACHE_DIR   Shared cache directory"
)]
pub(crate) struct Cli {
    #[arg(long, global = true, default_value_t = false)]
    pub(crate) json: bool,
    #[arg(long, global = true, default_value_t = false)]
    pub(crate) quiet: bool,
    #[arg(long, global = true, action = ArgAction::Count)]
    pub(crate) verbose: u8,
    #[arg(long, global = true, default_value_t = false)]
    pub(crate) trace: bool,
    #[arg(long = "bijux-plugin-metadata", default_value_t = false)]
    pub(crate) bijux_plugin_metadata: bool,
    #[arg(long = "print-config-paths", default_value_t = false)]
    pub(crate) print_config_paths: bool,
    #[arg(long = "umbrella-version")]
    pub(crate) umbrella_version: Option<String>,
    #[command(subcommand)]
    pub(crate) command: Option<AtlasCommand>,
}

#[derive(Subcommand)]
pub(crate) enum AtlasCommand {
    #[command(hide = true)]
    Validate {
        #[arg(long)]
        root: CliPathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long, default_value_t = false)]
        deep: bool,
    },
    Version,
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
    #[command(name = "config")]
    PrintConfig {
        #[arg(long, default_value_t = false)]
        canonical: bool,
    },
    Catalog {
        #[command(subcommand)]
        command: CatalogCommand,
    },
    Dataset {
        #[command(subcommand)]
        command: DatasetCommand,
    },
    Diff {
        #[command(subcommand)]
        command: DiffCommand,
    },
    Gc {
        #[command(subcommand)]
        command: GcCommand,
    },
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    Ingest {
        #[arg(long)]
        gff3: CliPathBuf,
        #[arg(long)]
        fasta: CliPathBuf,
        #[arg(long)]
        fai: CliPathBuf,
        #[arg(long, default_value_t = false)]
        allow_network_inputs: bool,
        #[arg(long, default_value_t = false)]
        resume: bool,
        #[arg(long)]
        output_root: CliPathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long, value_enum, default_value_t = StrictnessCli::Strict)]
        strictness: StrictnessCli,
        #[arg(long, value_enum, default_value_t = DuplicateGeneIdPolicyCli::Fail)]
        duplicate_gene_id_policy: DuplicateGeneIdPolicyCli,
        #[arg(long, value_enum, default_value_t = GeneIdentifierPolicyCli::Gff3Id)]
        gene_identifier_policy: GeneIdentifierPolicyCli,
        #[arg(long, default_value = "gene_id")]
        ensembl_keys: String,
        #[arg(long, default_value = "")]
        seqid_aliases: String,
        #[arg(long, default_value_t = 1)]
        max_threads: usize,
        #[arg(long, default_value_t = false)]
        report_only: bool,
        #[arg(long, default_value_t = false)]
        strict: bool,
        #[arg(long, default_value_t = false)]
        allow_overlap_gene_ids_across_contigs: bool,
        #[arg(long, default_value_t = false)]
        no_fai_check: bool,
        #[arg(long, default_value_t = false)]
        dev_auto_generate_fai: bool,
        #[arg(long, default_value_t = false)]
        fasta_scanning: bool,
        #[arg(long, default_value_t = 2000000000)]
        fasta_scan_max_bases: u64,
        #[arg(long, default_value_t = false)]
        emit_shards: bool,
        #[arg(long, default_value_t = 0)]
        shard_partitions: usize,
        #[arg(long, value_enum)]
        sharding_plan: Option<ShardingPlanCli>,
        #[arg(long, default_value_t = false)]
        emit_normalized_debug: bool,
        #[arg(long, default_value_t = false)]
        normalized_replay: bool,
        #[arg(long, default_value_t = false)]
        prod_mode: bool,
    },
    #[command(hide = true)]
    IngestVerifyInputs {
        #[arg(long)]
        gff3: CliPathBuf,
        #[arg(long)]
        fasta: CliPathBuf,
        #[arg(long)]
        fai: CliPathBuf,
        #[arg(long)]
        output_root: CliPathBuf,
        #[arg(long, default_value_t = false)]
        allow_network_inputs: bool,
        #[arg(long, default_value_t = false)]
        resume: bool,
    },
    #[command(hide = true)]
    IngestReplay {
        #[arg(long)]
        normalized: CliPathBuf,
    },
    #[command(hide = true)]
    IngestNormalizedDiff {
        #[arg(long)]
        base: CliPathBuf,
        #[arg(long)]
        target: CliPathBuf,
    },
    #[command(hide = true)]
    IngestValidate {
        #[arg(long)]
        qc_report: CliPathBuf,
        #[arg(long, default_value = "configs/sources/operations/ops/dataset-qc-thresholds.v1.json")]
        thresholds: CliPathBuf,
    },
    #[command(hide = true)]
    InspectDb {
        #[arg(long)]
        db: CliPathBuf,
        #[arg(long, default_value_t = 5)]
        sample_rows: usize,
    },
    #[command(hide = true)]
    ExplainQuery {
        #[arg(long)]
        db: CliPathBuf,
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
    #[command(hide = true)]
    Explain {
        #[arg(long)]
        db: CliPathBuf,
        #[arg(value_name = "QUERY")]
        query: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        allow_full_scan: bool,
    },
    #[command(hide = true)]
    Smoke {
        #[arg(long)]
        root: CliPathBuf,
        #[arg(long)]
        dataset: String,
        #[arg(
            long,
            default_value = "ops/datasets/fixtures/medium/api-list-queries.v1.json"
        )]
        golden_queries: CliPathBuf,
        #[arg(long, default_value_t = false)]
        write_snapshot: bool,
        #[arg(
            long,
            default_value = "ops/datasets/fixtures/medium/api-list-responses.v1.json"
        )]
        snapshot_out: CliPathBuf,
    },
    Openapi {
        #[command(subcommand)]
        command: OpenapiCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum OpenapiCommand {
    Generate {
        #[arg(long, default_value = "configs/generated/openapi/v1/openapi.json")]
        out: CliPathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum PolicyCommand {
    Validate,
    Explain {
        #[arg(long, value_enum)]
        mode: Option<PolicyModeCli>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum StrictnessCli {
    Strict,
    Compat,
    Lenient,
    ReportOnly,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum DuplicateGeneIdPolicyCli {
    Fail,
    Dedupe,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum GeneIdentifierPolicyCli {
    Gff3Id,
    Ensembl,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum ShardingPlanCli {
    None,
    Contig,
    RegionGrid,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum PolicyModeCli {
    Strict,
    Compat,
    Dev,
}
