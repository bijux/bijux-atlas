-- SSOT: bijux-atlas-ingest SQLite schema v4
PRAGMA journal_mode=WAL;
PRAGMA synchronous=OFF;
PRAGMA locking_mode=EXCLUSIVE;
PRAGMA temp_store=MEMORY;
PRAGMA cache_size=-32000;
PRAGMA page_size=4096;
PRAGMA mmap_size=268435456;

CREATE TABLE gene_summary (
  id INTEGER PRIMARY KEY,
  gene_id TEXT NOT NULL,
  name TEXT NOT NULL,
  name_normalized TEXT NOT NULL,
  biotype TEXT NOT NULL,
  seqid TEXT NOT NULL,
  start INTEGER NOT NULL,
  end INTEGER NOT NULL,
  transcript_count INTEGER NOT NULL,
  exon_count INTEGER NOT NULL DEFAULT 0,
  total_exon_span INTEGER NOT NULL DEFAULT 0,
  cds_present INTEGER NOT NULL DEFAULT 0,
  sequence_length INTEGER NOT NULL
) WITHOUT ROWID;

CREATE TABLE transcript_summary (
  id INTEGER PRIMARY KEY,
  transcript_id TEXT NOT NULL UNIQUE,
  parent_gene_id TEXT NOT NULL,
  transcript_type TEXT NOT NULL,
  biotype TEXT,
  seqid TEXT NOT NULL,
  start INTEGER NOT NULL,
  end INTEGER NOT NULL,
  exon_count INTEGER NOT NULL DEFAULT 0,
  total_exon_span INTEGER NOT NULL DEFAULT 0,
  cds_present INTEGER NOT NULL DEFAULT 0
) WITHOUT ROWID;

CREATE TABLE genes (
  id INTEGER PRIMARY KEY,
  gene_id TEXT NOT NULL,
  name TEXT NOT NULL,
  name_normalized TEXT NOT NULL,
  biotype TEXT NOT NULL,
  seqid TEXT NOT NULL,
  start INTEGER NOT NULL,
  end INTEGER NOT NULL,
  transcript_count INTEGER NOT NULL,
  exon_count INTEGER NOT NULL DEFAULT 0,
  total_exon_span INTEGER NOT NULL DEFAULT 0,
  cds_present INTEGER NOT NULL DEFAULT 0,
  sequence_length INTEGER NOT NULL
) WITHOUT ROWID;

CREATE TABLE transcripts (
  id INTEGER PRIMARY KEY,
  transcript_id TEXT NOT NULL UNIQUE,
  parent_gene_id TEXT NOT NULL,
  transcript_type TEXT NOT NULL,
  biotype TEXT,
  seqid TEXT NOT NULL,
  start INTEGER NOT NULL,
  end INTEGER NOT NULL,
  exon_count INTEGER NOT NULL DEFAULT 0,
  total_exon_span INTEGER NOT NULL DEFAULT 0,
  cds_present INTEGER NOT NULL DEFAULT 0,
  sequence_length INTEGER NOT NULL DEFAULT 0,
  spliced_length INTEGER,
  cds_length INTEGER
) WITHOUT ROWID;

CREATE TABLE exons (
  id INTEGER PRIMARY KEY,
  exon_id TEXT NOT NULL,
  transcript_id TEXT NOT NULL,
  seqid TEXT NOT NULL,
  start INTEGER NOT NULL,
  end INTEGER NOT NULL,
  exon_length INTEGER NOT NULL
) WITHOUT ROWID;

CREATE TABLE transcript_exon_map (
  transcript_id TEXT NOT NULL,
  exon_id TEXT NOT NULL,
  PRIMARY KEY (transcript_id, exon_id)
) WITHOUT ROWID;

CREATE TABLE atlas_meta (
  k TEXT PRIMARY KEY,
  v TEXT NOT NULL
) WITHOUT ROWID;

CREATE TABLE schema_version (
  version INTEGER PRIMARY KEY
) WITHOUT ROWID;

CREATE TABLE dataset_stats (
  dimension TEXT NOT NULL,
  value TEXT NOT NULL,
  gene_count INTEGER NOT NULL,
  PRIMARY KEY (dimension, value)
) WITHOUT ROWID;

CREATE TABLE contigs (
  name TEXT PRIMARY KEY,
  length INTEGER NOT NULL,
  gc_fraction REAL,
  n_fraction REAL
) WITHOUT ROWID;

CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(
  gene_rowid,
  start,
  end
);

CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
CREATE INDEX idx_gene_summary_name ON gene_summary(name);
CREATE INDEX idx_gene_summary_name_normalized ON gene_summary(name_normalized);
CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
CREATE INDEX idx_gene_summary_cover_lookup ON gene_summary(gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length);
CREATE INDEX idx_gene_summary_cover_region ON gene_summary(seqid, start, gene_id, end, name, biotype, transcript_count, sequence_length);

CREATE INDEX idx_transcript_summary_transcript_id ON transcript_summary(transcript_id);
CREATE INDEX idx_transcript_summary_parent_gene_id ON transcript_summary(parent_gene_id);
CREATE INDEX idx_transcript_summary_biotype ON transcript_summary(biotype);
CREATE INDEX idx_transcript_summary_type ON transcript_summary(transcript_type);
CREATE INDEX idx_transcript_summary_region ON transcript_summary(seqid, start, end);

CREATE INDEX idx_genes_gene_id ON genes(gene_id);
CREATE INDEX idx_genes_name ON genes(name);
CREATE INDEX idx_genes_biotype ON genes(biotype);
CREATE INDEX idx_genes_order_page ON genes(seqid, start, gene_id);

CREATE INDEX idx_transcripts_tx_id ON transcripts(transcript_id);
CREATE INDEX idx_transcripts_parent_gene ON transcripts(parent_gene_id);
CREATE INDEX idx_transcripts_order_page ON transcripts(seqid, start, transcript_id);

CREATE INDEX idx_exons_transcript ON exons(transcript_id);
CREATE INDEX idx_exons_region ON exons(seqid, start, end);
