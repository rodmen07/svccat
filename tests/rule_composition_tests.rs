use svccat::rules::{Rule, RuleEngine};

fn create_test_service(
    name: &str,
    platform: Option<&str>,
    team: Option<&str>,
    language: Option<&str>,
) -> svccat::manifest::ServiceEntry {
    svccat::manifest::ServiceEntry {
        name: name.to_string(),
        language: language.map(|s| s.to_string()),
        platform: platform.map(|s| s.to_string()),
        url: None,
        role: None,
        team: team.map(|s| s.to_string()),
        oncall: None,
        submodule: None,
        path: None,
        docs: None,
        ci: None,
        tags: vec!["critical".to_string()],
        depends_on: Vec::new(),
    }
}

#[test]
fn test_simple_rule_inheritance() {
    // Base rule requires team existence
    let base_rule = Rule {
        id: "base_team_requirement".to_string(),
        description: "Services must have a team".to_string(),
        expression: "team exists".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    // Override: derive from base but stricter expression
    let derived_rule = Rule {
        id: "critical_service_team".to_string(),
        description: "Critical services must have team and oncall".to_string(),
        expression: "team exists".to_string(), // Could be overridden
        severity: "error".to_string(),
        base: Some("base_team_requirement".to_string()),
    };

    let engine = RuleEngine::compile(&[base_rule, derived_rule]).expect("Should compile");

    let with_team = create_test_service("api", Some("Cloud Run"), Some("platform"), None);
    let without_team = create_test_service("api", Some("Cloud Run"), None, None);

    // Both rules should pass for service with team
    let violations_with_team = engine.evaluate(&with_team);
    assert_eq!(
        violations_with_team.len(),
        0,
        "Service with team should pass both rules"
    );

    // Both rules should fail for service without team
    let violations_without_team = engine.evaluate(&without_team);
    assert_eq!(
        violations_without_team.len(),
        2,
        "Service without team should fail both rules"
    );
}

#[test]
fn test_rule_inheritance_with_expression_override() {
    // Base rule: platforms in approved list
    let base_rule = Rule {
        id: "approved_platforms".to_string(),
        description: "Only approved platforms allowed".to_string(),
        expression: "platform in [Cloud Run, GKE]".to_string(),
        severity: "warning".to_string(),
        base: None,
    };

    // Derived rule: stricter set of platforms
    let derived_rule = Rule {
        id: "critical_platform_restriction".to_string(),
        description: "Critical services restricted to GKE only".to_string(),
        expression: "platform in [GKE]".to_string(), // Override with stricter requirement
        severity: "error".to_string(),
        base: Some("approved_platforms".to_string()),
    };

    let engine = RuleEngine::compile(&[base_rule, derived_rule]).expect("Should compile");

    let gke_service = create_test_service("api", Some("GKE"), None, None);
    let cloud_run_service = create_test_service("api", Some("Cloud Run"), None, None);
    let lambda_service = create_test_service("api", Some("Lambda"), None, None);

    // GKE: base passes, derived passes
    let violations_gke = engine.evaluate(&gke_service);
    assert_eq!(violations_gke.len(), 0, "GKE should pass both rules");

    // Cloud Run: base passes, derived fails
    let violations_cloud_run = engine.evaluate(&cloud_run_service);
    assert_eq!(
        violations_cloud_run.len(),
        1,
        "Cloud Run should fail stricter rule"
    );
    assert_eq!(
        violations_cloud_run[0].rule_id,
        "critical_platform_restriction"
    );

    // Lambda: base and derived both fail
    let violations_lambda = engine.evaluate(&lambda_service);
    assert_eq!(violations_lambda.len(), 2, "Lambda should fail both rules");
}

#[test]
fn test_rule_inheritance_chain() {
    // Level 1: Basic requirement
    let level1 = Rule {
        id: "has_platform".to_string(),
        description: "Service must have a platform".to_string(),
        expression: "platform exists".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    // Level 2: Adds strictness
    let level2 = Rule {
        id: "approved_platform".to_string(),
        description: "Platform must be from approved list".to_string(),
        expression: "platform in [Cloud Run, GKE, Lambda]".to_string(),
        severity: "warning".to_string(),
        base: Some("has_platform".to_string()),
    };

    // Level 3: Most strict
    let level3 = Rule {
        id: "critical_platform".to_string(),
        description: "Critical services need highly available platform".to_string(),
        expression: "platform in [GKE]".to_string(),
        severity: "error".to_string(),
        base: Some("approved_platform".to_string()),
    };

    let engine = RuleEngine::compile(&[level1, level2, level3]).expect("Should compile");

    let no_platform = create_test_service("api", None, None, None);
    let lambda_service = create_test_service("api", Some("Lambda"), None, None);
    let gke_service = create_test_service("api", Some("GKE"), None, None);

    // No platform: all fail (level1, level2 inherited, level3 inherited)
    let violations_none = engine.evaluate(&no_platform);
    assert_eq!(
        violations_none.len(),
        3,
        "Service without platform should fail all 3 rules"
    );

    // Lambda: level1 passes, level2 passes, level3 fails
    let violations_lambda = engine.evaluate(&lambda_service);
    assert_eq!(
        violations_lambda.len(),
        1,
        "Lambda should fail only the strictest rule"
    );

    // GKE: all pass
    let violations_gke = engine.evaluate(&gke_service);
    assert_eq!(violations_gke.len(), 0, "GKE should pass all rules");
}

#[test]
fn test_rule_inheritance_with_severity_override() {
    let base_rule = Rule {
        id: "naming_convention".to_string(),
        description: "Service name must match pattern".to_string(),
        expression: "name matches ^service-".to_string(),
        severity: "warning".to_string(),
        base: None,
    };

    // Derived: same rule but stricter severity
    let derived_rule = Rule {
        id: "critical_naming_convention".to_string(),
        description: "Critical service naming must match pattern".to_string(),
        expression: "name matches ^service-".to_string(),
        severity: "error".to_string(), // Override severity
        base: Some("naming_convention".to_string()),
    };

    let engine = RuleEngine::compile(&[base_rule, derived_rule]).expect("Should compile");

    let good_name = create_test_service("service-api", None, None, None);
    let bad_name = create_test_service("api", None, None, None);

    // Good name: passes both
    let violations_good = engine.evaluate(&good_name);
    assert_eq!(violations_good.len(), 0);

    // Bad name: fails both, check severity levels
    let violations_bad = engine.evaluate(&bad_name);
    assert_eq!(violations_bad.len(), 2);
    // One should be warning, one should be error
    let severities: Vec<_> = violations_bad.iter().map(|v| &v.severity).collect();
    assert!(severities.contains(&&"warning".to_string()));
    assert!(severities.contains(&&"error".to_string()));
}

#[test]
fn test_rule_inheritance_missing_base() {
    let rule_with_missing_base = Rule {
        id: "orphan_rule".to_string(),
        description: "This rule references a non-existent base".to_string(),
        expression: "team exists".to_string(),
        severity: "error".to_string(),
        base: Some("nonexistent_base".to_string()),
    };

    let result = RuleEngine::compile(&[rule_with_missing_base]);
    assert!(result.is_err(), "Should fail when base rule doesn't exist");
    assert!(result.unwrap_err().to_string().contains("Base rule"));
}

#[test]
fn test_multiple_rules_with_shared_base() {
    // Common base rule
    let base_rule = Rule {
        id: "has_documentation".to_string(),
        description: "Service must have documentation".to_string(),
        expression: "docs exists".to_string(),
        severity: "warning".to_string(),
        base: None,
    };

    // Multiple rules deriving from the same base
    let derived1 = Rule {
        id: "backend_documentation".to_string(),
        description: "Backend services must have docs".to_string(),
        expression: "docs exists".to_string(),
        severity: "error".to_string(),
        base: Some("has_documentation".to_string()),
    };

    let derived2 = Rule {
        id: "api_documentation".to_string(),
        description: "APIs must have docs".to_string(),
        expression: "docs exists".to_string(),
        severity: "error".to_string(),
        base: Some("has_documentation".to_string()),
    };

    let engine = RuleEngine::compile(&[base_rule, derived1, derived2]).expect("Should compile");

    let _with_docs = create_test_service("api", Some("Cloud Run"), None, None);
    let without_docs = create_test_service("api", Some("Cloud Run"), None, None);

    // Both derived rules use the same base - should both apply
    let violations_without = engine.evaluate(&without_docs);
    assert_eq!(
        violations_without.len(),
        3,
        "Should have 3 violations: 1 base + 2 derived"
    );
}

#[test]
fn test_rule_inheritance_empty_override() {
    // Base with expression
    let base = Rule {
        id: "base_rule".to_string(),
        description: "Base description".to_string(),
        expression: "team exists".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    // Derived without overriding anything (just inherits)
    let derived = Rule {
        id: "derived_rule".to_string(),
        description: "Derived description".to_string(), // This IS different, so it overrides
        expression: "team exists".to_string(),
        severity: "error".to_string(),
        base: Some("base_rule".to_string()),
    };

    let engine = RuleEngine::compile(&[base, derived]).expect("Should compile");

    let with_team = create_test_service("api", None, Some("platform"), None);
    let violations = engine.evaluate(&with_team);

    // Both rules should be present and passing
    assert_eq!(
        violations.len(),
        0,
        "Service with team should pass both rules"
    );
}

#[test]
fn test_rule_composition_with_boolean_expressions() {
    // Base: team requirement
    let base = Rule {
        id: "has_team".to_string(),
        description: "Must have a team".to_string(),
        expression: "team exists".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    // Derived: team AND specific platform
    let derived = Rule {
        id: "team_on_approved_platform".to_string(),
        description: "Team must be assigned to approved platform".to_string(),
        expression: "team exists AND platform in [Cloud Run, GKE]".to_string(),
        severity: "error".to_string(),
        base: Some("has_team".to_string()),
    };

    let engine = RuleEngine::compile(&[base, derived]).expect("Should compile");

    let good_service = create_test_service("api", Some("Cloud Run"), Some("platform"), None);
    let bad_platform = create_test_service("api", Some("Lambda"), Some("platform"), None);
    let no_team = create_test_service("api", Some("Cloud Run"), None, None);

    // Good service: passes both
    assert_eq!(engine.evaluate(&good_service).len(), 0);

    // Bad platform: base passes, derived fails
    assert_eq!(engine.evaluate(&bad_platform).len(), 1);

    // No team: both fail
    assert_eq!(engine.evaluate(&no_team).len(), 2);
}
