use svccat::rules::{Rule, RuleEngine};

fn create_test_service(
    name: &str,
    platform: Option<&str>,
    team: Option<&str>,
    language: Option<&str>,
) -> svccat::manifest::ServiceEntry {
    let mut svc = svccat::manifest::ServiceEntry::default();
    svc.name = name.to_string();
    svc.language = language.map(|s| s.to_string());
    svc.platform = platform.map(|s| s.to_string());
    svc.team = team.map(|s| s.to_string());
    svc.tags = vec!["critical".to_string()];
    svc
}

#[test]
fn test_simple_and_expression() {
    let rule = Rule {
        id: "test_and".to_string(),
        description: "Test AND operator".to_string(),
        expression: "name matches ^service- AND platform in [Cloud Run, GKE]".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let passing_service =
        create_test_service("service-api", Some("Cloud Run"), Some("platform"), None);
    let failing_service = create_test_service("api", Some("Cloud Run"), Some("platform"), None);

    let passing_violations = engine.evaluate(&passing_service);
    let failing_violations = engine.evaluate(&failing_service);

    assert_eq!(passing_violations.len(), 0, "Should pass AND expression");
    assert_eq!(failing_violations.len(), 1, "Should fail AND expression");
}

#[test]
fn test_simple_or_expression() {
    let rule = Rule {
        id: "test_or".to_string(),
        description: "Test OR operator".to_string(),
        expression: "platform in [Cloud Run] OR platform in [GKE]".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let cloud_run_service = create_test_service("api", Some("Cloud Run"), Some("platform"), None);
    let gke_service = create_test_service("api", Some("GKE"), Some("platform"), None);
    let lambda_service = create_test_service("api", Some("Lambda"), Some("platform"), None);

    assert_eq!(
        engine.evaluate(&cloud_run_service).len(),
        0,
        "Cloud Run should pass"
    );
    assert_eq!(engine.evaluate(&gke_service).len(), 0, "GKE should pass");
    assert_eq!(
        engine.evaluate(&lambda_service).len(),
        1,
        "Lambda should fail"
    );
}

#[test]
fn test_not_expression() {
    let rule = Rule {
        id: "test_not".to_string(),
        description: "Test NOT operator".to_string(),
        expression: "NOT name matches ^internal-".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let public_service = create_test_service("service-api", None, None, None);
    let internal_service = create_test_service("internal-tools", None, None, None);

    assert_eq!(
        engine.evaluate(&public_service).len(),
        0,
        "Public service should pass"
    );
    assert_eq!(
        engine.evaluate(&internal_service).len(),
        1,
        "Internal service should fail"
    );
}

#[test]
fn test_operator_precedence() {
    // Test: (critical AND team) OR platform matches Cloud
    // NOT should bind tighter than AND which binds tighter than OR
    let rule = Rule {
        id: "test_precedence".to_string(),
        description: "Test operator precedence".to_string(),
        expression: "team exists AND platform in [Cloud Run] OR platform in [GKE]".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let with_team_cloud = create_test_service("api", Some("Cloud Run"), Some("platform"), None);
    let with_team_lambda = create_test_service("api", Some("Lambda"), Some("platform"), None);
    let no_team_gke = create_test_service("api", Some("GKE"), None, None);

    assert_eq!(
        engine.evaluate(&with_team_cloud).len(),
        0,
        "Team + Cloud Run should pass"
    );
    assert_eq!(
        engine.evaluate(&with_team_lambda).len(),
        1,
        "Team + Lambda should fail"
    );
    assert_eq!(
        engine.evaluate(&no_team_gke).len(),
        0,
        "No team + GKE should pass (OR condition)"
    );
}

#[test]
fn test_parenthesized_expressions() {
    let rule = Rule {
        id: "test_parens".to_string(),
        description: "Test parenthesized expressions".to_string(),
        expression: "(name matches ^service- AND team exists) OR platform in [GKE]".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let service_with_team =
        create_test_service("service-api", Some("Lambda"), Some("platform"), None);
    let gke_no_team = create_test_service("api", Some("GKE"), None, None);
    let lambda_no_team = create_test_service("api", Some("Lambda"), None, None);

    assert_eq!(
        engine.evaluate(&service_with_team).len(),
        0,
        "Service name + team should pass"
    );
    assert_eq!(engine.evaluate(&gke_no_team).len(), 0, "GKE should pass");
    assert_eq!(
        engine.evaluate(&lambda_no_team).len(),
        1,
        "Neither condition met should fail"
    );
}

#[test]
fn test_complex_nested_expression() {
    let rule = Rule {
        id: "test_complex".to_string(),
        description: "Complex nested expression".to_string(),
        expression: "(name matches ^service- AND platform in [Cloud Run]) OR (team exists AND NOT name matches ^internal-)".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let service_cloud = create_test_service("service-api", Some("Cloud Run"), None, None);
    let team_public = create_test_service("worker", Some("Lambda"), Some("platform"), None);
    let team_internal =
        create_test_service("internal-tools", Some("Lambda"), Some("platform"), None);
    let neither = create_test_service("api", Some("Lambda"), None, None);

    assert_eq!(
        engine.evaluate(&service_cloud).len(),
        0,
        "Service + Cloud Run should pass"
    );
    assert_eq!(
        engine.evaluate(&team_public).len(),
        0,
        "Team + not internal should pass"
    );
    assert_eq!(
        engine.evaluate(&team_internal).len(),
        1,
        "Team + internal should fail"
    );
    assert_eq!(
        engine.evaluate(&neither).len(),
        1,
        "Neither condition should fail"
    );
}

#[test]
fn test_multiple_or_expressions() {
    let rule = Rule {
        id: "test_multi_or".to_string(),
        description: "Multiple OR conditions".to_string(),
        expression: "platform in [Cloud Run] OR platform in [GKE] OR platform in [Heroku]"
            .to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let cloud_run = create_test_service("api", Some("Cloud Run"), None, None);
    let gke = create_test_service("api", Some("GKE"), None, None);
    let heroku = create_test_service("api", Some("Heroku"), None, None);
    let lambda = create_test_service("api", Some("Lambda"), None, None);

    assert_eq!(
        engine.evaluate(&cloud_run).len(),
        0,
        "Cloud Run should pass"
    );
    assert_eq!(engine.evaluate(&gke).len(), 0, "GKE should pass");
    assert_eq!(engine.evaluate(&heroku).len(), 0, "Heroku should pass");
    assert_eq!(engine.evaluate(&lambda).len(), 1, "Lambda should fail");
}

#[test]
fn test_backward_compatibility() {
    // Old simple expressions should still work
    let rule = Rule {
        id: "test_compat".to_string(),
        description: "Backward compatibility test".to_string(),
        expression: "name matches ^service-".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let service = create_test_service("service-api", None, None, None);
    let other = create_test_service("api", None, None, None);

    assert_eq!(engine.evaluate(&service).len(), 0, "Should pass");
    assert_eq!(engine.evaluate(&other).len(), 1, "Should fail");
}

#[test]
fn test_invalid_expression_syntax() {
    let rule = Rule {
        id: "test_invalid".to_string(),
        description: "Invalid expression".to_string(),
        expression: "invalid expression syntax".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let result = RuleEngine::compile(&[rule]);
    assert!(result.is_err(), "Should reject invalid syntax");
}

#[test]
fn test_unclosed_parenthesis() {
    let rule = Rule {
        id: "test_unclosed".to_string(),
        description: "Unclosed parenthesis".to_string(),
        expression: "(name matches ^service-".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let result = RuleEngine::compile(&[rule]);
    assert!(result.is_err(), "Should reject unclosed parenthesis");
}

#[test]
fn test_double_negation() {
    let rule = Rule {
        id: "test_double_not".to_string(),
        description: "Double negation".to_string(),
        expression: "NOT NOT name matches ^service-".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let engine = RuleEngine::compile(&[rule]).expect("Should compile");

    let service = create_test_service("service-api", None, None, None);
    let other = create_test_service("api", None, None, None);

    assert_eq!(
        engine.evaluate(&service).len(),
        0,
        "Should pass (double NOT)"
    );
    assert_eq!(engine.evaluate(&other).len(), 1, "Should fail (double NOT)");
}
