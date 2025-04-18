use analogz::container::LogBuf;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_load_hdfs_log_file(c: &mut Criterion) {
    c.bench_function("LogBuf::new", |b| {
        b.iter(|| {
            LogBuf::new(std::fs::read_to_string("HDFS.log").unwrap());
        });
    });
}

criterion_group!(benches, bench_load_hdfs_log_file);
criterion_main!(benches);
