use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;

fn benchmark_dir_summary(c: &mut Criterion) {
    c.bench_function("DirSummary::scan /tmp", |b| {
        b.iter(|| cfm_lib::fs::DirSummary::scan(black_box(Path::new("/tmp"))).unwrap())
    });
}

fn benchmark_git_info(c: &mut Criterion) {
    c.bench_function("get_git_info /tmp", |b| {
        b.iter(|| cfm_lib::git::get_git_info(black_box(Path::new("/tmp"))).ok())
    });
}

fn benchmark_format_size(c: &mut Criterion) {
    c.bench_function("format_size_compact", |b| {
        b.iter(|| cfm_lib::fs::format_size_compact(black_box(1234567)))
    });
}

fn benchmark_format_exact_time(c: &mut Criterion) {
    c.bench_function("format_exact_time", |b| {
        let dt = chrono::Utc::now();
        b.iter(|| cfm_lib::fs::format_exact_time(black_box(&dt)))
    });
}

fn benchmark_format_relative_time(c: &mut Criterion) {
    c.bench_function("format_relative_time", |b| {
        let dt = chrono::Utc::now() - chrono::Duration::hours(2);
        b.iter(|| cfm_lib::fs::format_relative_time(black_box(&dt)))
    });
}

criterion_group!(
    benches,
    benchmark_dir_summary,
    benchmark_git_info,
    benchmark_format_size,
    benchmark_format_exact_time,
    benchmark_format_relative_time,
);
criterion_main!(benches);
