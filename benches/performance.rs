use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn bench_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join("fab_bench_dir");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..500 {
            let ext = match i % 5 {
                0 => "rs",
                1 => "md",
                2 => "toml",
                3 => "txt",
                _ => "json",
            };
            std::fs::write(dir.join(format!("file_{i:03}.{ext}")), b"benchmark").unwrap();
        }
        for i in 0..25 {
            let child = dir.join(format!("dir_{i:02}"));
            std::fs::create_dir_all(&child).unwrap();
            std::fs::write(child.join("child.md"), b"benchmark").unwrap();
        }
        dir
    })
}

fn benchmark_project_detect(c: &mut Criterion) {
    c.bench_function("ProjectType::detect temp", |b| {
        b.iter(|| folder_auto_banner::fs::ProjectType::detect(black_box(bench_dir())))
    });
}

fn benchmark_dir_summary(c: &mut Criterion) {
    c.bench_function("DirSummary::scan /tmp", |b| {
        b.iter(|| folder_auto_banner::fs::DirSummary::scan(black_box(Path::new("/tmp"))).unwrap())
    });
}

fn benchmark_git_info(c: &mut Criterion) {
    c.bench_function("get_git_info /tmp", |b| {
        b.iter(|| folder_auto_banner::git::get_git_info(black_box(Path::new("/tmp"))).ok())
    });
}

fn benchmark_format_size(c: &mut Criterion) {
    c.bench_function("format_size_compact", |b| {
        b.iter(|| folder_auto_banner::fs::format_size_compact(black_box(1234567)))
    });
}

fn benchmark_format_exact_time(c: &mut Criterion) {
    c.bench_function("format_exact_time", |b| {
        let dt = chrono::Utc::now();
        b.iter(|| folder_auto_banner::fs::format_exact_time(black_box(&dt)))
    });
}

fn benchmark_format_relative_time(c: &mut Criterion) {
    c.bench_function("format_relative_time", |b| {
        let dt = chrono::Utc::now() - chrono::Duration::hours(2);
        b.iter(|| folder_auto_banner::fs::format_relative_time(black_box(&dt)))
    });
}

criterion_group!(
    benches,
    benchmark_project_detect,
    benchmark_dir_summary,
    benchmark_git_info,
    benchmark_format_size,
    benchmark_format_exact_time,
    benchmark_format_relative_time,
);
criterion_main!(benches);
