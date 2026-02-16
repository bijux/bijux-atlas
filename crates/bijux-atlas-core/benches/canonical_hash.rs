use bijux_atlas_core::canonical;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

fn bench_stable_json_bytes(c: &mut Criterion) {
    let payload = json!({
        "release": "110",
        "species": "homo_sapiens",
        "assembly": "GRCh38",
        "stats": {
            "gene_count": 10_000,
            "transcript_count": 20_000,
            "contig_count": 24
        },
        "checksums": {
            "gff3": "a".repeat(64),
            "fasta": "b".repeat(64),
            "sqlite": "c".repeat(64)
        }
    });

    c.bench_function("stable_json_bytes", |b| {
        b.iter(|| canonical::stable_json_bytes(black_box(&payload)).expect("stable json"))
    });
}

fn bench_stable_json_hash(c: &mut Criterion) {
    let payload = json!({
        "release": "110",
        "species": "homo_sapiens",
        "assembly": "GRCh38",
        "gene_id": "ENSG000001",
        "name": "GENE_A",
        "biotype": "protein_coding"
    });

    c.bench_function("stable_json_hash_hex", |b| {
        b.iter(|| canonical::stable_json_hash_hex(black_box(&payload)).expect("stable hash"))
    });
}

criterion_group!(benches, bench_stable_json_bytes, bench_stable_json_hash);
criterion_main!(benches);
