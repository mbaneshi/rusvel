//! Cold-ish startup slice: SQLite open + migrations, and department registry load.
//! Offline; no network or Ollama.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rusvel_core::registry::DepartmentRegistry;
use std::path::Path;

fn bench_database_open_temp(c: &mut Criterion) {
    c.bench_function("database_open_temp_wal", |b| {
        b.iter(|| {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("boot_bench.db");
            let _db = rusvel_db::Database::open(black_box(&path)).unwrap();
        });
    });
}

fn bench_department_registry_load_missing(c: &mut Criterion) {
    c.bench_function("department_registry_load_missing_path", |b| {
        let path = Path::new("/nonexistent/departments.toml");
        b.iter(|| {
            let _reg = DepartmentRegistry::load(black_box(path));
        });
    });
}

criterion_group!(
    benches,
    bench_database_open_temp,
    bench_department_registry_load_missing
);
criterion_main!(benches);
