use criterion::{black_box, criterion_group, criterion_main, Criterion};
use svccat::manifest::Manifest;

/// Benchmark manifest loading from a typical service catalog
fn bench_manifest_load(c: &mut Criterion) {
    c.bench_function("load_manifest_small", |b| {
        b.iter(|| {
            let manifest_data = black_box(
                r#"
version: "1"
discovery:
  paths: ["services/*"]
services:
  - name: api
    language: Rust
    platform: "Cloud Run"
    url: "https://api.example.com"
    docs: "docs/README.md"
  - name: web
    language: TypeScript
    platform: "Cloud Run"
    url: "https://web.example.com"
    depends_on:
      - api
"#,
            );
            let _manifest: Manifest = serde_yaml::from_str(manifest_data).unwrap();
        })
    });

    c.bench_function("load_manifest_medium", |b| {
        b.iter(|| {
            let manifest_data =
                black_box(include_str!("../examples/sample-monorepo/services.yaml"));
            let _manifest: Manifest = serde_yaml::from_str(manifest_data).unwrap();
        })
    });
}

/// Benchmark URL validation (SSRF prevention)
fn bench_url_validation(c: &mut Criterion) {
    c.bench_function("validate_public_url", |b| {
        b.iter(|| {
            let url = black_box("https://api.example.com/endpoint");
            let result = svccat::urlvalidation::validate_url(url, false);
            let _ = black_box(result);
        })
    });

    c.bench_function("reject_private_ip", |b| {
        b.iter(|| {
            let url = black_box("http://127.0.0.1:8080");
            let result = svccat::urlvalidation::validate_url(url, false);
            let _ = black_box(result);
        })
    });

    c.bench_function("reject_ipv6_loopback", |b| {
        b.iter(|| {
            let url = black_box("http://[::1]:3000");
            let result = svccat::urlvalidation::validate_url(url, false);
            let _ = black_box(result);
        })
    });
}

/// Benchmark manifest dependency analysis
fn bench_git_ref_validation(c: &mut Criterion) {
    c.bench_function("analyze_dependencies", |b| {
        b.iter(|| {
            let manifest_data = black_box(
                r#"
version: "1"
discovery:
  paths: ["services/*"]
services:
  - name: api
    language: Rust
    depends_on: [web, auth]
  - name: web
    language: TypeScript
    depends_on: [api]
  - name: auth
    language: Python
    depends_on: []
"#,
            );
            let manifest: Manifest = serde_yaml::from_str(manifest_data).unwrap();
            let _ = black_box(manifest);
        })
    });
}

criterion_group!(
    benches,
    bench_manifest_load,
    bench_url_validation,
    bench_git_ref_validation
);
criterion_main!(benches);
