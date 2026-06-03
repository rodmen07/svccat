use std::path::{Path, PathBuf};
use svccat::drift;
use svccat::manifest::Manifest;

#[test]
fn test_rule_expression_parsing() {
    let rule = svccat::rules::Rule {
        id: "test_rule".to_string(),
        description: "Test rule".to_string(),
        expression: "name matches ^service-".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let result = svccat::rules::RuleEngine::compile(&[rule]);
    assert!(result.is_ok(), "Valid rule should compile successfully");
}

#[test]
fn test_invalid_rule_expression() {
    let rule = svccat::rules::Rule {
        id: "bad_rule".to_string(),
        description: "Invalid rule".to_string(),
        expression: "invalid expression syntax".to_string(),
        severity: "error".to_string(),
        base: None,
    };

    let result = svccat::rules::RuleEngine::compile(&[rule]);
    assert!(result.is_err(), "Invalid rule should fail to compile");
}

#[test]
fn test_rule_evaluation_with_manifest() {
    let manifest_path = PathBuf::from("tests/fixtures/rules/manifest-basic.yaml");

    let manifest = Manifest::load(&manifest_path).expect("Failed to load test manifest");

    // Verify the manifest has rules defined
    assert!(
        !manifest.policy.rules.is_empty(),
        "Manifest should have rules defined"
    );
    assert_eq!(manifest.policy.rules.len(), 3, "Should have 3 custom rules");

    // Compile the rule engine
    let engine = svccat::rules::RuleEngine::compile(&manifest.policy.rules)
        .expect("Failed to compile rules");

    // Test rule evaluation on each service
    for svc in &manifest.services {
        let violations = engine.evaluate(svc);

        match svc.name.as_str() {
            "service-api" => {
                // service-api passes naming convention and has team
                assert!(
                    violations.iter().all(|v| v.rule_id != "naming_convention"),
                    "service-api should pass naming convention"
                );
                assert!(
                    violations.iter().all(|v| v.rule_id != "required_team"),
                    "service-api should have team"
                );
            }
            "invalid-service" => {
                // invalid-service fails naming convention and has no team
                let naming_violations: Vec<_> = violations
                    .iter()
                    .filter(|v| v.rule_id == "naming_convention")
                    .collect();
                assert!(
                    !naming_violations.is_empty(),
                    "invalid-service should fail naming convention"
                );

                let team_violations: Vec<_> = violations
                    .iter()
                    .filter(|v| v.rule_id == "required_team")
                    .collect();
                assert!(
                    !team_violations.is_empty(),
                    "invalid-service should have no team"
                );
            }
            "service-auth" => {
                // service-auth passes naming convention but has no team
                assert!(
                    violations.iter().all(|v| v.rule_id != "naming_convention"),
                    "service-auth should pass naming convention"
                );
                let team_violations: Vec<_> = violations
                    .iter()
                    .filter(|v| v.rule_id == "required_team")
                    .collect();
                assert!(
                    !team_violations.is_empty(),
                    "service-auth should have no team"
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test_rule_violations_in_drift_report() {
    let manifest_path = PathBuf::from("tests/fixtures/rules/manifest-basic.yaml");

    let manifest = Manifest::load(&manifest_path).expect("Failed to load test manifest");

    let discovered = Vec::new(); // Empty discovered services

    let report = drift::analyze(&manifest, &discovered, Path::new("tests/fixtures/rules"));

    // Should have policy violations from custom rules
    let policy_violations: Vec<_> = report
        .drifts
        .iter()
        .filter(|d| d.kind == drift::DriftKind::PolicyViolation)
        .collect();

    assert!(
        !policy_violations.is_empty(),
        "Should have policy violations from custom rules"
    );

    // invalid-service should have multiple violations
    let invalid_violations: Vec<_> = policy_violations
        .iter()
        .filter(|d| d.service == "invalid-service")
        .collect();
    assert!(
        invalid_violations.len() >= 2,
        "invalid-service should have at least 2 violations"
    );
}

#[test]
fn test_rule_severity_levels() {
    let rules = vec![
        svccat::rules::Rule {
            id: "error_rule".to_string(),
            description: "This is an error rule".to_string(),
            expression: "name matches ^error-".to_string(),
            severity: "error".to_string(),
            base: None,
        },
        svccat::rules::Rule {
            id: "warning_rule".to_string(),
            description: "This is a warning rule".to_string(),
            expression: "name matches ^warn-".to_string(),
            severity: "warning".to_string(),
            base: None,
        },
    ];

    let engine = svccat::rules::RuleEngine::compile(&rules).expect("Failed to compile rules");

    let warn_service = svccat::manifest::ServiceEntry {
        name: "warn-test".to_string(),
        language: None,
        platform: None,
        role: None,
        team: None,
        url: None,
        oncall: None,
        submodule: None,
        path: None,
        docs: None,
        ci: None,
        tags: Vec::new(),
        depends_on: Vec::new(),
    };

    let violations = engine.evaluate(&warn_service);

    // Should have one violation from error_rule
    let error_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.severity == "error")
        .collect();
    assert_eq!(error_violations.len(), 1, "Should have 1 error violation");

    // Should have no violation from warning_rule (name matches warn-)
    let warning_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.severity == "warning")
        .collect();
    assert_eq!(
        warning_violations.len(),
        0,
        "Should have 0 warning violations"
    );
}

#[test]
fn test_multiple_rules_on_service() {
    let manifest_path = PathBuf::from("tests/fixtures/rules/manifest-basic.yaml");

    let manifest = Manifest::load(&manifest_path).expect("Failed to load test manifest");

    let engine = svccat::rules::RuleEngine::compile(&manifest.policy.rules)
        .expect("Failed to compile rules");

    // Find invalid-service which should violate multiple rules
    let invalid_service = manifest
        .services
        .iter()
        .find(|s| s.name == "invalid-service")
        .expect("Should find invalid-service");

    let violations = engine.evaluate(invalid_service);

    // Should have violations for naming_convention and required_team
    let rule_ids: Vec<_> = violations.iter().map(|v| v.rule_id.as_str()).collect();
    assert!(
        rule_ids.contains(&"naming_convention"),
        "Should have naming_convention violation"
    );
    assert!(
        rule_ids.contains(&"required_team"),
        "Should have required_team violation"
    );
}

#[test]
fn test_platform_in_list_rule() {
    let rules = vec![svccat::rules::Rule {
        id: "approved_platforms".to_string(),
        description: "Only approved platforms".to_string(),
        expression: "platform in [Cloud Run, GKE, Heroku]".to_string(),
        severity: "warning".to_string(),
        base: None,
    }];

    let engine = svccat::rules::RuleEngine::compile(&rules).expect("Failed to compile rules");

    // Approved platform
    let approved_service = svccat::manifest::ServiceEntry {
        name: "test-service".to_string(),
        language: None,
        platform: Some("Cloud Run".to_string()),
        role: None,
        team: None,
        url: None,
        oncall: None,
        submodule: None,
        path: None,
        docs: None,
        ci: None,
        tags: Vec::new(),
        depends_on: Vec::new(),
    };

    let violations = engine.evaluate(&approved_service);
    assert_eq!(violations.len(), 0, "Approved platform should pass");

    // Unapproved platform
    let unapproved_service = svccat::manifest::ServiceEntry {
        name: "test-service".to_string(),
        language: None,
        platform: Some("Lambda".to_string()),
        role: None,
        team: None,
        url: None,
        oncall: None,
        submodule: None,
        path: None,
        docs: None,
        ci: None,
        tags: Vec::new(),
        depends_on: Vec::new(),
    };

    let violations = engine.evaluate(&unapproved_service);
    assert_eq!(violations.len(), 1, "Unapproved platform should fail");
}
