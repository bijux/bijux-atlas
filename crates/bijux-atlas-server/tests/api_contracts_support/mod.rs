use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, DatasetId, GeneId, ManifestStats, ReleaseGeneIndex,
    ReleaseGeneIndexEntry, SeqId,
};
use rusqlite::Connection;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn fixture_sqlite() -> Vec<u8> {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("x.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, exon_count INT DEFAULT 0, total_exon_span INT DEFAULT 0, cds_present INT DEFAULT 0, sequence_length INT);
         CREATE TABLE transcript_summary(id INTEGER PRIMARY KEY, transcript_id TEXT, parent_gene_id TEXT, transcript_type TEXT, biotype TEXT, seqid TEXT, start INT, end INT, exon_count INT, total_exon_span INT, cds_present INT);
         CREATE TABLE dataset_stats(dimension TEXT NOT NULL, value TEXT NOT NULL, gene_count INTEGER NOT NULL, PRIMARY KEY (dimension, value));
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10,1,10);
         CREATE INDEX idx_transcript_summary_transcript_id ON transcript_summary(transcript_id);
         CREATE INDEX idx_transcript_summary_parent_gene_id ON transcript_summary(parent_gene_id);
         CREATE INDEX idx_transcript_summary_biotype ON transcript_summary(biotype);
         CREATE INDEX idx_transcript_summary_type ON transcript_summary(transcript_type);
         CREATE INDEX idx_transcript_summary_region ON transcript_summary(seqid,start,end);
         INSERT INTO transcript_summary(id,transcript_id,parent_gene_id,transcript_type,biotype,seqid,start,end,exon_count,total_exon_span,cds_present) VALUES (1,'tx1','g1','transcript','pc','chr1',1,10,1,10,1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('biotype','pc',1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('seqid','chr1',1);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}

pub fn mk_dataset() -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let (fasta, fai) = fixture_fasta_and_fai();
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        ArtifactChecksums::new(
            "a".repeat(64),
            sha256_hex(&fasta),
            sha256_hex(&fai),
            sqlite_sha,
        ),
        ManifestStats::new(1, 1, 1),
    );
    (ds, manifest, sqlite)
}

pub fn fixture_fasta_and_fai() -> (Vec<u8>, Vec<u8>) {
    let fasta = b">chr1\nACGTACGTAC\nGGGGnnnnTT\n".to_vec();
    let fai = b"chr1\t20\t6\t10\t11\n".to_vec();
    (fasta, fai)
}

pub fn fixture_release_index(
    dataset: &DatasetId,
    rows: Vec<(&str, &str, u64, u64, &str)>,
) -> Vec<u8> {
    let mut entries: Vec<ReleaseGeneIndexEntry> = rows
        .into_iter()
        .map(|(gene_id, seqid, start, end, sig)| {
            ReleaseGeneIndexEntry::new(
                GeneId::parse(gene_id).expect("gene id"),
                SeqId::parse(seqid).expect("seqid"),
                start,
                end,
                sig.to_string(),
            )
        })
        .collect();
    entries.sort();
    serde_json::to_vec(&ReleaseGeneIndex::new(
        "1".to_string(),
        dataset.clone(),
        entries,
    ))
    .expect("index json")
}

pub async fn send_raw(
    addr: std::net::SocketAddr,
    path: &str,
    headers: &[(&str, &str)],
) -> (u16, String, String) {
    send_raw_with_method(addr, "GET", path, headers, None).await
}

pub async fn send_raw_with_method(
    addr: std::net::SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect server");
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    if let Some(payload) = body {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", payload.len()));
    }
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    if let Some(payload) = body {
        req.push_str(payload);
    }
    stream
        .write_all(req.as_bytes())
        .await
        .expect("write request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .expect("read response");
    let (head, body) = response
        .split_once("\r\n\r\n")
        .expect("http response must have separator");
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .expect("http status");
    (status, head.to_string(), body.to_string())
}
