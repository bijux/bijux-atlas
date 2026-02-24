use bijux_atlas_model::{GeneId, ReleaseGeneIndexEntry, SeqId};
use criterion::{criterion_group, criterion_main, Criterion};

fn build_entries(prefix: &str, n: usize, changed_every: usize) -> Vec<ReleaseGeneIndexEntry> {
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(ReleaseGeneIndexEntry::new(
            GeneId::parse(&format!("{prefix}{i:07}")).expect("gene id"),
            SeqId::parse("chr1").expect("seqid"),
            (i as u64) * 10 + 1,
            (i as u64) * 10 + 5,
            format!("sig-{}", i / changed_every),
        ));
    }
    out
}

fn diff_merge_count(a: &[ReleaseGeneIndexEntry], b: &[ReleaseGeneIndexEntry]) -> usize {
    let mut i = 0usize;
    let mut j = 0usize;
    let mut count = 0usize;
    while i < a.len() || j < b.len() {
        match (a.get(i), b.get(j)) {
            (Some(x), Some(y)) => match x.gene_id.cmp(&y.gene_id) {
                std::cmp::Ordering::Less => {
                    i += 1;
                    count += 1;
                }
                std::cmp::Ordering::Greater => {
                    j += 1;
                    count += 1;
                }
                std::cmp::Ordering::Equal => {
                    i += 1;
                    j += 1;
                    if x.signature_sha256 != y.signature_sha256 {
                        count += 1;
                    }
                }
            },
            (Some(_), None) => {
                i += 1;
                count += 1;
            }
            (None, Some(_)) => {
                j += 1;
                count += 1;
            }
            (None, None) => break,
        }
    }
    count
}

fn bench_diff_merge(c: &mut Criterion) {
    let mut from = build_entries("g", 50_000, 5);
    let mut to = build_entries("g", 50_000, 7);
    to.extend(build_entries("n", 1_000, 3));
    from.extend(build_entries("r", 900, 3));
    from.sort();
    to.sort();

    c.bench_function("diff_merge_50k", |b| {
        b.iter(|| {
            let changed = diff_merge_count(&from, &to);
            assert!(changed > 0);
        })
    });
}

criterion_group!(benches, bench_diff_merge);
criterion_main!(benches);
