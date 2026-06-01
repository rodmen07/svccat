use crate::manifest::ServiceEntry;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub id: String,
    pub description: String,
    pub expression: String,
    pub severity: String,
}

#[derive(Debug)]
pub enum RuleExpression {
    NameMatches(Regex),
    FieldExists(String),
    FieldMatches(String, Regex),
    FieldIn(String, Vec<String>),
    And(Vec<RuleExpression>),
    Or(Vec<RuleExpression>),
}

pub struct RuleEngine {
    rules: Vec<(Rule, RuleExpression)>,
}

#[derive(Debug)]
pub struct RuleViolation {
    pub rule_id: String,
    pub service_name: String,
    pub severity: String,
    pub message: String,
}

impl RuleEngine {
    pub fn compile(rules: &[Rule]) -> Result<Self> {
        let mut compiled = Vec::new();
        for rule in rules {
            let expr = parse_expression(&rule.expression)?;
            compiled.push((rule.clone(), expr));
        }
        Ok(RuleEngine { rules: compiled })
    }

    pub fn evaluate(&self, service: &ServiceEntry) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        for (rule, expr) in &self.rules {
            if !evaluate_expression(expr, service) {
                violations.push(RuleViolation {
                    rule_id: rule.id.clone(),
                    service_name: service.name.clone(),
                    severity: rule.severity.clone(),
                    message: rule.description.clone(),
                });
            }
        }
        violations
    }
}

fn parse_expression(expr_str: &str) -> Result<RuleExpression> {
    let expr = expr_str.trim();

    if let Some(pattern) = expr.strip_prefix("name matches ") {
        let pattern = pattern.trim_matches(|c| c == '"' || c == '\'');
        let regex = Regex::new(pattern)
            .map_err(|e| anyhow!("Invalid regex pattern in rule: {}", e))?;
        return Ok(RuleExpression::NameMatches(regex));
    }

    if let Some(rest) = expr.strip_prefix("name in [") {
        if let Some(values_str) = rest.strip_suffix("]") {
            let values: Vec<String> = values_str
                .split(',')
                .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                .collect();
            return Ok(RuleExpression::FieldIn("name".to_string(), values));
        }
    }

    if let Some(_) = expr.strip_prefix("team exists") {
        return Ok(RuleExpression::FieldExists("team".to_string()));
    }

    if let Some(_) = expr.strip_prefix("team != null") {
        return Ok(RuleExpression::FieldExists("team".to_string()));
    }

    if let Some(rest) = expr.strip_prefix("platform in [") {
        if let Some(values_str) = rest.strip_suffix("]") {
            let values: Vec<String> = values_str
                .split(',')
                .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                .collect();
            return Ok(RuleExpression::FieldIn("platform".to_string(), values));
        }
    }

    if let Some(rest) = expr.strip_prefix("language matches ") {
        let pattern = rest.trim_matches(|c| c == '"' || c == '\'');
        let regex = Regex::new(pattern)
            .map_err(|e| anyhow!("Invalid regex pattern in rule: {}", e))?;
        return Ok(RuleExpression::FieldMatches("language".to_string(), regex));
    }

    Err(anyhow!("Unsupported rule expression: {}", expr))
}

fn evaluate_expression(expr: &RuleExpression, service: &ServiceEntry) -> bool {
    match expr {
        RuleExpression::NameMatches(regex) => regex.is_match(&service.name),

        RuleExpression::FieldExists(field_name) => {
            get_field_value(service, field_name).is_some()
        }

        RuleExpression::FieldMatches(field_name, regex) => {
            if let Some(value) = get_field_value(service, field_name) {
                regex.is_match(&value)
            } else {
                false
            }
        }

        RuleExpression::FieldIn(field_name, allowed_values) => {
            if let Some(value) = get_field_value(service, field_name) {
                allowed_values.iter().any(|v| v == &value)
            } else {
                false
            }
        }

        RuleExpression::And(exprs) => exprs.iter().all(|e| evaluate_expression(e, service)),

        RuleExpression::Or(exprs) => exprs.iter().any(|e| evaluate_expression(e, service)),
    }
}

fn get_field_value(service: &ServiceEntry, field_name: &str) -> Option<String> {
    match field_name {
        "name" => Some(service.name.clone()),
        "language" => service.language.clone(),
        "platform" => service.platform.clone(),
        "team" => service.team.clone(),
        "role" => service.role.clone(),
        "url" => service.url.clone(),
        "oncall" => service.oncall.clone(),
        "docs" => service.docs.clone(),
        "ci" => service.ci.clone(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service(name: &str, platform: &str, team: Option<&str>) -> ServiceEntry {
        ServiceEntry {
            name: name.to_string(),
            language: Some("Rust".to_string()),
            platform: Some(platform.to_string()),
            role: Some("Service".to_string()),
            team: team.map(|t| t.to_string()),
            url: None,
            oncall: None,
            submodule: None,
            path: None,
            docs: None,
            ci: None,
            tags: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    #[test]
    fn test_name_matches_pattern() {
        let service = create_test_service("api-service", "Cloud Run", Some("platform"));
        let rule = Rule {
            id: "naming_convention".to_string(),
            description: "Services must match pattern".to_string(),
            expression: "name matches ^api-".to_string(),
            severity: "error".to_string(),
        };

        let engine = RuleEngine::compile(&[rule]).unwrap();
        let violations = engine.evaluate(&service);
        assert_eq!(violations.len(), 0, "Service should pass name pattern");
    }

    #[test]
    fn test_name_matches_failure() {
        let service = create_test_service("my-service", "Cloud Run", Some("platform"));
        let rule = Rule {
            id: "naming_convention".to_string(),
            description: "Services must match pattern".to_string(),
            expression: "name matches ^api-".to_string(),
            severity: "error".to_string(),
        };

        let engine = RuleEngine::compile(&[rule]).unwrap();
        let violations = engine.evaluate(&service);
        assert_eq!(violations.len(), 1, "Service should fail name pattern");
        assert_eq!(violations[0].rule_id, "naming_convention");
    }

    #[test]
    fn test_field_exists() {
        let service = create_test_service("api-service", "Cloud Run", Some("platform"));
        let rule = Rule {
            id: "required_team".to_string(),
            description: "Services must have a team".to_string(),
            expression: "team exists".to_string(),
            severity: "error".to_string(),
        };

        let engine = RuleEngine::compile(&[rule]).unwrap();
        let violations = engine.evaluate(&service);
        assert_eq!(violations.len(), 0, "Service should pass team exists check");
    }

    #[test]
    fn test_field_exists_failure() {
        let service = create_test_service("api-service", "Cloud Run", None);
        let rule = Rule {
            id: "required_team".to_string(),
            description: "Services must have a team".to_string(),
            expression: "team exists".to_string(),
            severity: "error".to_string(),
        };

        let engine = RuleEngine::compile(&[rule]).unwrap();
        let violations = engine.evaluate(&service);
        assert_eq!(violations.len(), 1, "Service should fail team exists check");
    }

    #[test]
    fn test_field_in_list() {
        let service = create_test_service("api-service", "Cloud Run", Some("platform"));
        let rule = Rule {
            id: "approved_platforms".to_string(),
            description: "Only approved platforms".to_string(),
            expression: "platform in [Cloud Run, GKE, Heroku]".to_string(),
            severity: "warning".to_string(),
        };

        let engine = RuleEngine::compile(&[rule]).unwrap();
        let violations = engine.evaluate(&service);
        assert_eq!(violations.len(), 0, "Service should pass platform check");
    }

    #[test]
    fn test_field_in_list_failure() {
        let service = create_test_service("api-service", "Lambda", Some("platform"));
        let rule = Rule {
            id: "approved_platforms".to_string(),
            description: "Only approved platforms".to_string(),
            expression: "platform in [Cloud Run, GKE, Heroku]".to_string(),
            severity: "warning".to_string(),
        };

        let engine = RuleEngine::compile(&[rule]).unwrap();
        let violations = engine.evaluate(&service);
        assert_eq!(violations.len(), 1, "Service should fail platform check");
    }

    #[test]
    fn test_invalid_expression() {
        let rule = Rule {
            id: "bad_rule".to_string(),
            description: "Invalid rule".to_string(),
            expression: "invalid expression syntax".to_string(),
            severity: "error".to_string(),
        };

        let result = RuleEngine::compile(&[rule]);
        assert!(result.is_err(), "Should reject invalid expression");
    }
}
