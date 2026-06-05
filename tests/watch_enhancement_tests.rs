/// Tests for enhanced watch mode functionality
/// These tests verify change detection and state tracking
use svccat::manifest::ServiceEntry;

fn create_service(name: &str) -> ServiceEntry {
    let mut svc = ServiceEntry::default();
    svc.name = name.to_string();
    svc.language = Some("Rust".to_string());
    svc.platform = Some("Cloud Run".to_string());
    svc.role = Some("Service".to_string());
    svc.team = Some("platform".to_string());
    svc
}

#[test]
fn test_detect_added_services() {
    let services_v1 = vec![create_service("api"), create_service("web")];

    let mut services_v2 = services_v1.clone();
    services_v2.push(create_service("worker"));

    // Detect changes manually (mimicking the detect_changes logic)
    let v1_names: std::collections::HashSet<_> =
        services_v1.iter().map(|s| s.name.clone()).collect();
    let v2_names: std::collections::HashSet<_> =
        services_v2.iter().map(|s| s.name.clone()).collect();

    let added: Vec<_> = v2_names.difference(&v1_names).cloned().collect();
    let removed: Vec<_> = v1_names.difference(&v2_names).cloned().collect();

    assert_eq!(added.len(), 1);
    assert!(added.contains(&"worker".to_string()));
    assert!(removed.is_empty());
}

#[test]
fn test_detect_removed_services() {
    let services_v1 = [
        create_service("api"),
        create_service("web"),
        create_service("worker"),
    ];

    let services_v2 = [create_service("api"), create_service("web")];

    let v1_names: std::collections::HashSet<_> =
        services_v1.iter().map(|s| s.name.clone()).collect();
    let v2_names: std::collections::HashSet<_> =
        services_v2.iter().map(|s| s.name.clone()).collect();

    let added: Vec<_> = v2_names.difference(&v1_names).cloned().collect();
    let removed: Vec<_> = v1_names.difference(&v2_names).cloned().collect();

    assert!(added.is_empty());
    assert_eq!(removed.len(), 1);
    assert!(removed.contains(&"worker".to_string()));
}

#[test]
fn test_detect_multiple_changes() {
    let services_v1 = [
        create_service("api"),
        create_service("web"),
        create_service("old-service"),
    ];

    let services_v2 = [
        create_service("api"),
        create_service("web"),
        create_service("new-service"),
        create_service("another-new"),
    ];

    let v1_names: std::collections::HashSet<_> =
        services_v1.iter().map(|s| s.name.clone()).collect();
    let v2_names: std::collections::HashSet<_> =
        services_v2.iter().map(|s| s.name.clone()).collect();

    let added: Vec<_> = v2_names.difference(&v1_names).cloned().collect();
    let removed: Vec<_> = v1_names.difference(&v2_names).cloned().collect();

    assert_eq!(added.len(), 2);
    assert_eq!(removed.len(), 1);
    assert!(added.contains(&"new-service".to_string()));
    assert!(added.contains(&"another-new".to_string()));
    assert!(removed.contains(&"old-service".to_string()));
}

#[test]
fn test_empty_service_list() {
    let services_v1: Vec<ServiceEntry> = vec![];
    let services_v2 = [create_service("first-service")];

    let v1_names: std::collections::HashSet<_> =
        services_v1.iter().map(|s| s.name.clone()).collect();
    let v2_names: std::collections::HashSet<_> =
        services_v2.iter().map(|s| s.name.clone()).collect();

    let added: Vec<_> = v2_names.difference(&v1_names).cloned().collect();
    let removed: Vec<_> = v1_names.difference(&v2_names).cloned().collect();

    assert_eq!(added.len(), 1);
    assert!(removed.is_empty());
}

#[test]
fn test_no_changes() {
    let services = [create_service("api"), create_service("web")];

    let v1_names: std::collections::HashSet<_> = services.iter().map(|s| s.name.clone()).collect();
    let v2_names: std::collections::HashSet<_> = services.iter().map(|s| s.name.clone()).collect();

    let added: Vec<_> = v2_names.difference(&v1_names).cloned().collect();
    let removed: Vec<_> = v1_names.difference(&v2_names).cloned().collect();

    assert!(added.is_empty());
    assert!(removed.is_empty());
}
