use analogz::{container::LogBuf, tokenizer::Tokenize};
use criterion::{Criterion, criterion_group, criterion_main};
use rayon::iter::{ParallelBridge, ParallelIterator};

fn bench_tokenize(c: &mut Criterion) {
    c.bench_function("Tokenize::tokenize", |b| {
        b.iter(|| {
            LogBuf::new(std::fs::read_to_string("HDFS.log").unwrap())
                .iter()
                .par_bridge()
                .flat_map(|line| line.as_str().tokenize().par_bridge())
                .collect::<Vec<_>>();
        });
    });
}

criterion_group!(benches, bench_tokenize);
criterion_main!(benches);
