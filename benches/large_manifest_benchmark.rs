use criterion::{black_box, criterion_group, criterion_main, Criterion};
use svccat::manifest::Manifest;

/// Generate a manifest with N services for benchmarking scaling behavior
fn generate_large_manifest(service_count: usize) -> String {
    let mut yaml = String::from(
        r#"version: "1"
discovery:
  paths: ["services/*"]
services:
"#,
    );

    for i in 0..service_count {
        yaml.push_str(&format!(
            r#"  - name: service-{:04}
    language: Rust
    platform: Cloud Run
    url: "https://service-{}.example.com"
    role: "Service {}"
    team: team-{}
    docs: "docs/service-{:04}.md"
    depends_on:
      - "service-{:04}"
"#,
            i,
            i,
            i,
            i % 5,
            i,
            (i + 1) % service_count
        ));
    }

    yaml
}

fn bench_large_manifest_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_manifest");
    group.sample_size(10);

    // Benchmark parsing different manifest sizes
    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(format!("{}_services", size)),
            size,
            |b, &size| {
                b.iter(|| {
                    let manifest_data = black_box(generate_large_manifest(size));
                    let _manifest: Manifest = serde_yaml::from_str(&manifest_data).unwrap();
                })
            },
        );
    }

    group.finish();
}

fn bench_large_manifest_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_manifest_discovery");
    group.sample_size(10);

    // Simulate discovery-like filtering on large manifests
    for size in [100, 500, 1000].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(format!("{}_services", size)),
            size,
            |b, &size| {
                let manifest_data = generate_large_manifest(size);
                let manifest: Manifest = serde_yaml::from_str(&manifest_data).unwrap();

                b.iter(|| {
                    // Simulate service lookup by name (common operation in drift detection)
                    let _names: Vec<&str> =
                        manifest.services.iter().map(|s| s.name.as_str()).collect();
                    let _found = manifest.services.iter().any(|s| s.name == "service-0500");
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_large_manifest_parsing,
    bench_large_manifest_discovery
);
criterion_main!(benches);
