use svccat::deps_graph::{DependencyGraph, ServiceKey};
use svccat::manifest::Manifest;

fn create_test_service(
    name: &str,
    language: &str,
    platform: &str,
    depends_on: Vec<&str>,
) -> svccat::manifest::ServiceEntry {
    let mut svc = svccat::manifest::ServiceEntry::default();
    svc.name = name.to_string();
    svc.language = Some(language.to_string());
    svc.platform = Some(platform.to_string());
    svc.role = Some("Service".to_string());
    svc.team = Some("platform".to_string());
    svc.depends_on = depends_on.iter().map(|s| s.to_string()).collect();
    svc
}

fn create_test_manifest(services: Vec<svccat::manifest::ServiceEntry>) -> Manifest {
    let mut manifest = Manifest::default();
    manifest.version = "1".to_string();
    manifest.discovery = svccat::manifest::DiscoveryConfig {
        paths: vec!["services/*".to_string()],
        markers: vec![
            "package.json".to_string(),
            "Cargo.toml".to_string(),
            "go.mod".to_string(),
        ],
        ignore: Vec::new(),
    };
    manifest.services = services;
    manifest
}

#[test]
fn test_simple_cross_repo_dependencies() {
    let backend_manifest = create_test_manifest(vec![
        create_test_service(
            "api",
            "Rust",
            "Cloud Run",
            vec!["backend:auth", "backend:db"],
        ),
        create_test_service("auth", "Rust", "Cloud Run", vec![]),
        create_test_service("db", "Postgres", "Cloud SQL", vec![]),
    ]);

    let frontend_manifest = create_test_manifest(vec![create_test_service(
        "web",
        "TypeScript",
        "Cloud Run",
        vec!["backend:api"],
    )]);

    let graph = DependencyGraph::build(vec![
        ("backend".to_string(), &backend_manifest),
        ("frontend".to_string(), &frontend_manifest),
    ])
    .expect("Should build dependency graph");

    // Verify web depends on backend:api
    let web_key = ServiceKey {
        repo: "frontend".to_string(),
        service: "web".to_string(),
    };

    let web_deps = graph.find_dependencies(&web_key);
    assert!(!web_deps.is_empty(), "web should have dependencies");
    assert_eq!(web_deps[0].repo, "backend");
    assert_eq!(web_deps[0].service, "api");
}

#[test]
fn test_circular_dependency_detection() {
    let manifest = create_test_manifest(vec![
        create_test_service("api", "Rust", "Cloud Run", vec!["auth"]),
        create_test_service("auth", "Rust", "Cloud Run", vec!["api"]),
    ]);

    let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)])
        .expect("Should build dependency graph");

    assert!(
        !graph.circular_dependencies.is_empty(),
        "Should detect circular dependencies"
    );
}

#[test]
fn test_find_dependents() {
    let backend_manifest = create_test_manifest(vec![
        create_test_service("api", "Rust", "Cloud Run", vec!["backend:auth"]),
        create_test_service("worker", "Rust", "Cloud Run", vec!["backend:auth"]),
        create_test_service("auth", "Rust", "Cloud Run", vec![]),
    ]);

    let graph = DependencyGraph::build(vec![("backend".to_string(), &backend_manifest)])
        .expect("Should build dependency graph");

    let auth_key = ServiceKey {
        repo: "backend".to_string(),
        service: "auth".to_string(),
    };

    let dependents = graph.find_dependents(&auth_key);
    assert_eq!(
        dependents.len(),
        2,
        "auth should have 2 dependents (api and worker)"
    );
}

#[test]
fn test_impact_analysis() {
    let backend_manifest = create_test_manifest(vec![
        create_test_service("api", "Rust", "Cloud Run", vec!["backend:auth"]),
        create_test_service("web", "TypeScript", "Cloud Run", vec!["backend:api"]),
        create_test_service("auth", "Rust", "Cloud Run", vec![]),
    ]);

    let graph = DependencyGraph::build(vec![("backend".to_string(), &backend_manifest)])
        .expect("Should build dependency graph");

    let auth_key = ServiceKey {
        repo: "backend".to_string(),
        service: "auth".to_string(),
    };

    let impact = graph.get_impact(&auth_key);

    // If auth changes, it affects api and transitively web
    assert!(
        impact.total_affected > 0,
        "auth change should have some impact"
    );
}

#[test]
fn test_unresolvable_dependencies() {
    let manifest = create_test_manifest(vec![
        create_test_service("api", "Rust", "Cloud Run", vec!["missing-service"]),
        create_test_service("auth", "Rust", "Cloud Run", vec![]),
    ]);

    let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)])
        .expect("Should build dependency graph");

    let unresolvable = graph.validate_all_dependencies();
    assert!(
        !unresolvable.is_empty(),
        "Should detect unresolvable dependency on missing-service"
    );
}

#[test]
fn test_cross_repo_unresolvable() {
    let backend_manifest = create_test_manifest(vec![create_test_service(
        "api",
        "Rust",
        "Cloud Run",
        vec!["frontend:web"],
    )]);

    let frontend_manifest = create_test_manifest(vec![create_test_service(
        "admin",
        "TypeScript",
        "Cloud Run",
        vec![],
    )]);

    let graph = DependencyGraph::build(vec![
        ("backend".to_string(), &backend_manifest),
        ("frontend".to_string(), &frontend_manifest),
    ])
    .expect("Should build dependency graph");

    let unresolvable = graph.validate_all_dependencies();
    assert!(
        !unresolvable.is_empty(),
        "Should detect unresolvable cross-repo dependency"
    );
}

#[test]
fn test_dependency_summary() {
    let backend_manifest = create_test_manifest(vec![
        create_test_service(
            "api",
            "Rust",
            "Cloud Run",
            vec!["backend:auth", "backend:db"],
        ),
        create_test_service("auth", "Rust", "Cloud Run", vec![]),
        create_test_service("db", "Postgres", "Cloud SQL", vec![]),
    ]);

    let frontend_manifest = create_test_manifest(vec![create_test_service(
        "web",
        "TypeScript",
        "Cloud Run",
        vec!["backend:api"],
    )]);

    let graph = DependencyGraph::build(vec![
        ("backend".to_string(), &backend_manifest),
        ("frontend".to_string(), &frontend_manifest),
    ])
    .expect("Should build dependency graph");

    let summary = graph.summary();

    assert_eq!(summary.total_services, 4);
    assert_eq!(summary.total_dependencies, 3); // api->auth, api->db, web->api
    assert_eq!(summary.cross_repo_dependencies, 1); // web->api
}

#[test]
fn test_same_repo_dependencies() {
    let manifest = create_test_manifest(vec![
        create_test_service("api", "Rust", "Cloud Run", vec!["auth"]),
        create_test_service("auth", "Rust", "Cloud Run", vec![]),
    ]);

    let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)])
        .expect("Should build dependency graph");

    let summary = graph.summary();

    assert_eq!(summary.total_services, 2);
    assert_eq!(summary.total_dependencies, 1);
    assert_eq!(summary.cross_repo_dependencies, 0);
}

#[test]
fn test_multiple_circular_dependencies() {
    let manifest = create_test_manifest(vec![
        create_test_service("a", "Rust", "Cloud Run", vec!["b"]),
        create_test_service("b", "Rust", "Cloud Run", vec!["c"]),
        create_test_service("c", "Rust", "Cloud Run", vec!["a"]),
        create_test_service("d", "Rust", "Cloud Run", vec!["e"]),
        create_test_service("e", "Rust", "Cloud Run", vec!["d"]),
    ]);

    let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)])
        .expect("Should build dependency graph");

    assert!(
        !graph.circular_dependencies.is_empty(),
        "Should detect circular dependencies"
    );
}

#[test]
fn test_deeply_nested_dependencies() {
    let manifest = create_test_manifest(vec![
        create_test_service("layer1", "Rust", "Cloud Run", vec!["layer2"]),
        create_test_service("layer2", "Rust", "Cloud Run", vec!["layer3"]),
        create_test_service("layer3", "Rust", "Cloud Run", vec!["layer4"]),
        create_test_service("layer4", "Rust", "Cloud Run", vec![]),
    ]);

    let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)])
        .expect("Should build dependency graph");

    let layer1_key = ServiceKey {
        repo: "backend".to_string(),
        service: "layer1".to_string(),
    };

    let layer4_key = ServiceKey {
        repo: "backend".to_string(),
        service: "layer4".to_string(),
    };

    let layer1_deps = graph.find_dependencies(&layer1_key);
    assert!(!layer1_deps.is_empty());

    // layer4 is depended on by layer3
    let layer4_dependents = graph.find_dependents(&layer4_key);
    assert!(!layer4_dependents.is_empty());
    assert_eq!(layer4_dependents[0].service, "layer3");
}
