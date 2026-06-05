use svccat::manifest::ServiceEntry;
use svccat::rules::{Rule, RuleEngine};

fn create_service(
    name: &str,
    language: Option<&str>,
    platform: Option<&str>,
    team: Option<&str>,
    oncall: Option<&str>,
) -> ServiceEntry {
    let mut svc = ServiceEntry::default();
    svc.name = name.to_string();
    svc.language = language.map(|s| s.to_string());
    svc.platform = platform.map(|s| s.to_string());
    svc.team = team.map(|s| s.to_string());
    svc.oncall = oncall.map(|s| s.to_string());
    svc
}

#[test]
fn test_complex_policy_with_inheritance_and_operators() {
    // Scenario: Complex microservices policy with multiple inheritance levels
    let rules = vec![
        // Level 1: Basic requirements
        Rule {
            id: "has_platform".to_string(),
            description: "All services must declare a platform".to_string(),
            expression: "platform exists".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        // Level 2: Approved platforms
        Rule {
            id: "approved_platform".to_string(),
            description: "Platform must be from approved list".to_string(),
            expression: "platform in [Cloud Run, GKE, Lambda]".to_string(),
            severity: "warning".to_string(),
            base: Some("has_platform".to_string()),
        },
        // Level 3: Critical services have stricter requirements
        Rule {
            id: "critical_platform".to_string(),
            description: "Critical services require highly available platform".to_string(),
            expression: "platform in [GKE]".to_string(),
            severity: "error".to_string(),
            base: Some("approved_platform".to_string()),
        },
        // Independent rule with boolean operators
        Rule {
            id: "critical_team_oncall".to_string(),
            description: "Critical services must have team and oncall".to_string(),
            expression: "team exists AND oncall exists".to_string(),
            severity: "error".to_string(),
            base: None,
        },
    ];

    let engine = RuleEngine::compile(&rules).expect("Should compile");

    // Test 1: Service with no platform
    let no_platform = create_service("service-api", Some("Rust"), None, Some("platform"), None);
    let violations = engine.evaluate(&no_platform);
    assert_eq!(violations.len(), 4, "Should fail all 4 rules");

    // Test 2: Service with Lambda platform (approved, but not critical)
    let lambda_service = create_service(
        "service-api",
        Some("Rust"),
        Some("Lambda"),
        Some("platform"),
        None,
    );
    let violations = engine.evaluate(&lambda_service);
    assert_eq!(violations.len(), 2, "Should fail critical rules");
    assert!(violations.iter().any(|v| v.rule_id == "critical_platform"));
    assert!(violations
        .iter()
        .any(|v| v.rule_id == "critical_team_oncall"));

    // Test 3: Service with GKE and team/oncall (fully compliant)
    let gke_complete = create_service(
        "service-api",
        Some("Rust"),
        Some("GKE"),
        Some("platform"),
        Some("oncall"),
    );
    let violations = engine.evaluate(&gke_complete);
    assert_eq!(violations.len(), 0, "Should pass all rules");
}

#[test]
fn test_rules_with_negation_and_inheritance() {
    let rules = vec![
        // Base rule: not internal
        Rule {
            id: "public_service".to_string(),
            description: "Public services should not be internal".to_string(),
            expression: "NOT name matches ^internal-".to_string(),
            severity: "warning".to_string(),
            base: None,
        },
        // Derived: stricter requirement for critical services
        Rule {
            id: "critical_not_internal".to_string(),
            description: "Critical services must not be internal and must have team".to_string(),
            expression: "NOT name matches ^internal- AND team exists".to_string(),
            severity: "error".to_string(),
            base: Some("public_service".to_string()),
        },
    ];

    let engine = RuleEngine::compile(&rules).expect("Should compile");

    // Public service with team: passes both
    let good = create_service("service-api", None, None, Some("platform"), None);
    let violations = engine.evaluate(&good);
    assert_eq!(violations.len(), 0);

    // Internal service: fails both
    let internal = create_service("internal-tools", None, None, Some("platform"), None);
    let violations = engine.evaluate(&internal);
    assert_eq!(violations.len(), 2);

    // Internal service without team: fails both
    let internal_no_team = create_service("internal-tools", None, None, None, None);
    let violations = engine.evaluate(&internal_no_team);
    assert_eq!(violations.len(), 2);
}

#[test]
fn test_rules_with_parentheses_and_precedence() {
    let rules = vec![
        Rule {
            id: "standard_requirement".to_string(),
            description: "Standard requirement".to_string(),
            expression: "(language in [Rust, Go] AND platform in [GKE]) OR platform in [Cloud Run]".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        // Override with different grouping
        Rule {
            id: "strict_requirement".to_string(),
            description: "Stricter requirement".to_string(),
            expression: "(language in [Rust] AND platform in [GKE]) OR (language in [Go] AND platform in [Cloud Run])".to_string(),
            severity: "error".to_string(),
            base: Some("standard_requirement".to_string()),
        },
    ];

    let engine = RuleEngine::compile(&rules).expect("Should compile");

    // Rust + GKE: passes both
    let rust_gke = create_service("api", Some("Rust"), Some("GKE"), None, None);
    assert_eq!(engine.evaluate(&rust_gke).len(), 0);

    // Go + Cloud Run: passes both
    let go_cloud = create_service("worker", Some("Go"), Some("Cloud Run"), None, None);
    assert_eq!(engine.evaluate(&go_cloud).len(), 0);

    // Rust + Cloud Run: passes standard, fails strict
    let rust_cloud = create_service("api", Some("Rust"), Some("Cloud Run"), None, None);
    assert_eq!(engine.evaluate(&rust_cloud).len(), 1);

    // Go + GKE: passes standard, fails strict
    let go_gke = create_service("worker", Some("Go"), Some("GKE"), None, None);
    assert_eq!(engine.evaluate(&go_gke).len(), 1);

    // Python + Lambda: fails both
    let python_lambda = create_service("script", Some("Python"), Some("Lambda"), None, None);
    assert_eq!(engine.evaluate(&python_lambda).len(), 2);
}

#[test]
fn test_multiple_inheritance_branches() {
    // Scenario: Different rule hierarchies for different aspects
    let rules = vec![
        // Platform hierarchy
        Rule {
            id: "has_platform".to_string(),
            description: "Must have platform".to_string(),
            expression: "platform exists".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        Rule {
            id: "approved_platform".to_string(),
            description: "Approved platforms".to_string(),
            expression: "platform in [Cloud Run, GKE]".to_string(),
            severity: "warning".to_string(),
            base: Some("has_platform".to_string()),
        },
        // Team hierarchy (independent)
        Rule {
            id: "has_team".to_string(),
            description: "Must have team".to_string(),
            expression: "team exists".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        Rule {
            id: "team_with_oncall".to_string(),
            description: "Team must have oncall".to_string(),
            expression: "team exists AND oncall exists".to_string(),
            severity: "warning".to_string(),
            base: Some("has_team".to_string()),
        },
        // Language hierarchy
        Rule {
            id: "has_language".to_string(),
            description: "Must declare language".to_string(),
            expression: "language exists".to_string(),
            severity: "warning".to_string(),
            base: None,
        },
        Rule {
            id: "approved_language".to_string(),
            description: "Only approved languages".to_string(),
            expression: "language in [Rust, Go, Python]".to_string(),
            severity: "error".to_string(),
            base: Some("has_language".to_string()),
        },
    ];

    let engine = RuleEngine::compile(&rules).expect("Should compile");

    // Fully compliant service
    let perfect = create_service(
        "service-api",
        Some("Rust"),
        Some("GKE"),
        Some("platform"),
        Some("oncall"),
    );
    assert_eq!(engine.evaluate(&perfect).len(), 0);

    // Missing oncall (fails team_with_oncall)
    let no_oncall = create_service(
        "service-api",
        Some("Rust"),
        Some("GKE"),
        Some("platform"),
        None,
    );
    let violations = engine.evaluate(&no_oncall);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule_id, "team_with_oncall");

    // Unapproved language (fails approved_language)
    let node_service = create_service(
        "web-app",
        Some("Node.js"),
        Some("GKE"),
        Some("platform"),
        Some("oncall"),
    );
    let violations = engine.evaluate(&node_service);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule_id, "approved_language");

    // Multiple issues
    let problematic = create_service("unknown", Some("Cobol"), Some("Lambda"), None, None);
    let violations = engine.evaluate(&problematic);
    // Should fail: platform (Lambda not approved), language (Cobol not approved), team (missing), team_with_oncall
    assert_eq!(violations.len(), 4); // approved_platform, approved_language, has_team, team_with_oncall
}

#[test]
fn test_severity_levels_in_complex_scenarios() {
    let rules = vec![
        // Warning level
        Rule {
            id: "warning_rule".to_string(),
            description: "Recommended but not required".to_string(),
            expression: "docs exists".to_string(),
            severity: "warning".to_string(),
            base: None,
        },
        // Error level
        Rule {
            id: "error_rule".to_string(),
            description: "Must have".to_string(),
            expression: "team exists".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        // Derived with escalated severity
        Rule {
            id: "critical_error".to_string(),
            description: "Critical version of error rule".to_string(),
            expression: "team exists".to_string(),
            severity: "error".to_string(),
            base: Some("error_rule".to_string()),
        },
    ];

    let engine = RuleEngine::compile(&rules).expect("Should compile");

    let no_docs_no_team = create_service("api", None, None, None, None);
    let violations = engine.evaluate(&no_docs_no_team);

    // Should have violations with both severity levels
    let warnings = violations
        .iter()
        .filter(|v| v.severity == "warning")
        .collect::<Vec<_>>();
    let errors = violations
        .iter()
        .filter(|v| v.severity == "error")
        .collect::<Vec<_>>();

    assert_eq!(warnings.len(), 1); // docs_exists
    assert_eq!(errors.len(), 2); // error_rule, critical_error
}

#[test]
fn test_complex_boolean_logic_scenarios() {
    let rules = vec![
        Rule {
            id: "flexible_requirement".to_string(),
            description: "Multiple ways to satisfy requirement".to_string(),
            expression: "(language in [Rust] AND platform in [Cloud Run]) OR (language in [Go] AND platform in [GKE]) OR (language in [Python] AND platform in [Lambda])".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        // Negation with boolean
        Rule {
            id: "no_unapproved".to_string(),
            description: "Cannot use unapproved combinations".to_string(),
            expression: "NOT (language in [Ruby, PHP] AND platform in [Cloud Run])".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        // Double negation
        Rule {
            id: "double_check".to_string(),
            description: "Definitely not internal".to_string(),
            expression: "NOT NOT name matches ^service-".to_string(),
            severity: "warning".to_string(),
            base: None,
        },
    ];

    let engine = RuleEngine::compile(&rules).expect("Should compile");

    // Valid combination: Rust + Cloud Run
    let valid1 = create_service("api", Some("Rust"), Some("Cloud Run"), None, None);
    let violations = engine.evaluate(&valid1);
    assert_eq!(
        violations
            .iter()
            .filter(|v| v.rule_id == "flexible_requirement")
            .count(),
        0
    );

    // Valid combination: Go + GKE
    let valid2 = create_service("worker", Some("Go"), Some("GKE"), None, None);
    let violations = engine.evaluate(&valid2);
    assert_eq!(
        violations
            .iter()
            .filter(|v| v.rule_id == "flexible_requirement")
            .count(),
        0
    );

    // Invalid combination: Rust + GKE
    let invalid = create_service("api", Some("Rust"), Some("GKE"), None, None);
    let violations = engine.evaluate(&invalid);
    assert_eq!(
        violations
            .iter()
            .filter(|v| v.rule_id == "flexible_requirement")
            .count(),
        1
    );

    // Unapproved combination: Ruby + Cloud Run (should fail)
    let unapproved = create_service("web", Some("Ruby"), Some("Cloud Run"), None, None);
    let violations = engine.evaluate(&unapproved);
    assert_eq!(
        violations
            .iter()
            .filter(|v| v.rule_id == "no_unapproved")
            .count(),
        1
    );

    // Double negation test: service-api should pass
    let service_api = create_service("service-api", None, None, None, None);
    let violations = engine.evaluate(&service_api);
    assert_eq!(
        violations
            .iter()
            .filter(|v| v.rule_id == "double_check")
            .count(),
        0
    );
}

#[test]
fn test_error_handling_in_complex_expressions() {
    // Test that invalid inherited rules are caught
    let rules = vec![Rule {
        id: "orphan".to_string(),
        description: "References non-existent base".to_string(),
        expression: "team exists".to_string(),
        severity: "error".to_string(),
        base: Some("nonexistent".to_string()),
    }];

    let result = RuleEngine::compile(&rules);
    assert!(result.is_err(), "Should reject orphaned base reference");
    assert!(result.unwrap_err().to_string().contains("Base rule"));
}

#[test]
fn test_all_rule_features_combined() {
    // Ultimate integration test: all features working together
    let rules = vec![
        // Base rules
        Rule {
            id: "base_platform".to_string(),
            description: "Platform exists".to_string(),
            expression: "platform exists".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        Rule {
            id: "base_language".to_string(),
            description: "Language exists".to_string(),
            expression: "language exists".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        // Inheritance with expression override
        Rule {
            id: "approved_platform".to_string(),
            description: "Platform must be approved".to_string(),
            expression: "platform in [Cloud Run, GKE]".to_string(),
            severity: "warning".to_string(),
            base: Some("base_platform".to_string()),
        },
        Rule {
            id: "approved_language".to_string(),
            description: "Language must be approved".to_string(),
            expression: "language in [Rust, Go, Python]".to_string(),
            severity: "warning".to_string(),
            base: Some("base_language".to_string()),
        },
        // Boolean operators with inheritance
        Rule {
            id: "critical_approved".to_string(),
            description: "Critical services need approved stack".to_string(),
            expression: "platform in [GKE] AND language in [Rust, Go]".to_string(),
            severity: "error".to_string(),
            base: Some("approved_platform".to_string()),
        },
        // Negation
        Rule {
            id: "not_deprecated".to_string(),
            description: "Cannot use deprecated languages".to_string(),
            expression: "NOT language in [PHP, Ruby]".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        // Complex boolean
        Rule {
            id: "team_governance".to_string(),
            description: "Public services need team, private don't".to_string(),
            expression: "(NOT name matches ^internal- AND team exists) OR name matches ^internal-".to_string(),
            severity: "warning".to_string(),
            base: None,
        },
        // Parentheses + inheritance + operators
        Rule {
            id: "production_ready".to_string(),
            description: "Production ready services".to_string(),
            expression: "(platform in [GKE] OR platform in [Cloud Run]) AND (language in [Rust, Go]) AND NOT name matches ^experimental-".to_string(),
            severity: "error".to_string(),
            base: None,
        },
    ];

    let engine = RuleEngine::compile(&rules).expect("Should compile all features");

    // Production service: fully compliant
    let prod = create_service(
        "service-api",
        Some("Rust"),
        Some("GKE"),
        Some("platform"),
        None,
    );
    let violations = engine.evaluate(&prod);
    // Should only fail team_governance (no oncall/team check in this rule)
    assert!(violations.is_empty() || violations.iter().all(|v| v.rule_id == "team_governance"));

    // Experimental service: fails production_ready
    let exp = create_service(
        "experimental-feature",
        Some("Go"),
        Some("GKE"),
        Some("platform"),
        None,
    );
    let violations = engine.evaluate(&exp);
    assert!(violations.iter().any(|v| v.rule_id == "production_ready"));

    // Internal service: should pass team_governance differently
    let internal = create_service("internal-tools", Some("Go"), Some("GKE"), None, None);
    let violations = engine.evaluate(&internal);
    // Should not fail team_governance since it's internal
    assert!(violations
        .iter()
        .all(|v| v.rule_id != "team_governance" || !v.message.contains("team")));

    // Deprecated language: fails
    let old_lang = create_service(
        "legacy",
        Some("PHP"),
        Some("Cloud Run"),
        Some("platform"),
        None,
    );
    let violations = engine.evaluate(&old_lang);
    assert!(violations.iter().any(|v| v.rule_id == "not_deprecated"));
}
