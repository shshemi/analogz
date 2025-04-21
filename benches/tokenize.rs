use analogz::{container::Buffer, tokenizer::Tokenize};
use criterion::{Criterion, criterion_group, criterion_main};
use rayon::iter::{ParallelBridge, ParallelIterator};

fn bench_tokenize(c: &mut Criterion) {
    c.bench_function("Tokenize::tokenize", |b| {
        b.iter(|| {
            Buffer::new(std::fs::read_to_string("target/HDFS.log").unwrap())
                .iter()
                .par_bridge()
                .flat_map(|line| line.tokenize().par_bridge())
                .collect::<Vec<_>>();
        });
    });
}

criterion_group!(benches, bench_tokenize);
criterion_main!(benches);
