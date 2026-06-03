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
    /// Optional: inherit from another rule and override specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub id: String,
    pub description: String,
    pub expr: RuleExpression,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub enum RuleExpression {
    NameMatches(Regex),
    FieldExists(String),
    FieldMatches(String, Regex),
    FieldIn(String, Vec<String>),
    Contains(String, String),
    NotEqual(String, String),
    Equal(String, String),
    And(Box<RuleExpression>, Box<RuleExpression>),
    Or(Box<RuleExpression>, Box<RuleExpression>),
    Not(Box<RuleExpression>),
}

#[derive(Debug)]
pub struct RuleEngine {
    rules: Vec<CompiledRule>,
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
        // First pass: resolve inheritance and build rule map
        let mut rule_map: std::collections::HashMap<String, Rule> =
            std::collections::HashMap::new();
        for rule in rules {
            rule_map.insert(rule.id.clone(), rule.clone());
        }

        // Second pass: compile rules with inheritance
        let mut compiled = Vec::new();
        for rule in rules {
            let resolved = Self::resolve_rule(rule, &rule_map)?;
            let severity = match resolved.severity.to_lowercase().as_str() {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                s => {
                    return Err(anyhow!(
                        "Invalid severity level: {} (must be 'error' or 'warning')",
                        s
                    ))
                }
            };
            let expr = parse_expression(&resolved.expression)?;
            compiled.push(CompiledRule {
                id: resolved.id,
                description: resolved.description,
                expr,
                severity,
            });
        }
        Ok(RuleEngine { rules: compiled })
    }

    /// Resolve a rule by following its inheritance chain
    fn resolve_rule(
        rule: &Rule,
        rule_map: &std::collections::HashMap<String, Rule>,
    ) -> Result<Rule> {
        if let Some(base_id) = &rule.base {
            let base_rule = rule_map.get(base_id).ok_or_else(|| {
                anyhow!("Base rule '{}' not found for rule '{}'", base_id, rule.id)
            })?;

            // Recursively resolve base rule to handle chains
            let mut resolved_base = Self::resolve_rule(base_rule, rule_map)?;

            // Override base with current rule's fields (if they differ from defaults)
            if !rule.description.is_empty() && rule.description != base_rule.description {
                resolved_base.description = rule.description.clone();
            }
            if !rule.expression.is_empty() && rule.expression != base_rule.expression {
                resolved_base.expression = rule.expression.clone();
            }
            if !rule.severity.is_empty() && rule.severity != base_rule.severity {
                resolved_base.severity = rule.severity.clone();
            }
            resolved_base.id = rule.id.clone();
            Ok(resolved_base)
        } else {
            Ok(rule.clone())
        }
    }

    pub fn evaluate(&self, service: &ServiceEntry) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        for rule in &self.rules {
            if !evaluate_expression(&rule.expr, service) {
                violations.push(RuleViolation {
                    rule_id: rule.id.clone(),
                    service_name: service.name.clone(),
                    severity: match rule.severity {
                        Severity::Error => "error".to_string(),
                        Severity::Warning => "warning".to_string(),
                    },
                    message: rule.description.clone(),
                });
            }
        }
        violations
    }
}

// ── Advanced Expression Parser with Operator Precedence ──────────────────

fn parse_expression(expr_str: &str) -> Result<RuleExpression> {
    // Tokenize respecting parentheses and brackets
    let mut tokens = Vec::new();
    let mut remaining = expr_str.trim();

    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        // Handle parentheses
        if remaining.starts_with('(') {
            tokens.push(Token::LParen);
            remaining = &remaining[1..];
            continue;
        }
        if remaining.starts_with(')') {
            tokens.push(Token::RParen);
            remaining = &remaining[1..];
            continue;
        }

        // Handle NOT operator (already have space after, so just check prefix)
        if remaining.to_uppercase().starts_with("NOT ") {
            tokens.push(Token::Not);
            remaining = &remaining[4..];
            continue;
        }

        // Check for AND/OR at the start (already have space after, so just check prefix)
        let upper = remaining.to_uppercase();
        if upper.starts_with("AND ") {
            tokens.push(Token::And);
            remaining = &remaining[4..];
            continue;
        }
        if upper.starts_with("OR ") {
            tokens.push(Token::Or);
            remaining = &remaining[3..];
            continue;
        }

        // Parse atomic expression - find where it ends (before AND/OR/paren at the same level)
        let (token, rest) = parse_atomic_with_boundaries(remaining)?;
        tokens.push(token);
        remaining = rest.trim_start();
    }

    tokens.push(Token::Eof);
    let mut parser = Parser { tokens, pos: 0 };
    parser.parse()
}

fn parse_atomic_with_boundaries(s: &str) -> Result<(Token, &str)> {
    // Find the natural boundary of an atomic expression
    // (stop at AND, OR, or closing paren at top level)

    let mut bracket_depth = 0;
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut byte_end = s.len();

    for (i, c) in s.char_indices() {
        // Handle quotes
        if (c == '"' || c == '\'')
            && (i == 0 || s.chars().nth(i.saturating_sub(1)).unwrap_or(' ') != '\\')
        {
            if in_quote && c == quote_char {
                in_quote = false;
            } else if !in_quote {
                in_quote = true;
                quote_char = c;
            }
            continue;
        }

        // Only look for operators outside quotes and brackets
        if !in_quote {
            if c == '[' {
                bracket_depth += 1;
            } else if c == ']' {
                bracket_depth -= 1;
            } else if bracket_depth == 0 {
                // Check for AND/OR preceded by space
                if c.is_whitespace() {
                    let rest = &s[i..];
                    let rest_upper = rest.to_uppercase();

                    // Check for " AND " or " AND" at end
                    if rest_upper.starts_with(" AND ") || rest_upper == " AND" {
                        byte_end = i;
                        break;
                    }
                    // Check for " OR " or " OR" at end
                    if rest_upper.starts_with(" OR ") || rest_upper == " OR" {
                        byte_end = i;
                        break;
                    }
                }
                // Also stop at unescaped parentheses
                if c == ')' || c == '(' {
                    byte_end = i;
                    break;
                }
            }
        }
    }

    let atomic_str = s[..byte_end].trim_end();
    let rest = &s[byte_end..];

    if atomic_str.is_empty() {
        return Err(anyhow!("Empty atomic expression"));
    }

    let expr = parse_atomic_expression(atomic_str)?;
    Ok((Token::Atomic(expr), rest))
}

#[derive(Debug, Clone)]
enum Token {
    Atomic(RuleExpression),
    And,
    Or,
    Not,
    LParen,
    RParen,
    Eof,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn parse(&mut self) -> Result<RuleExpression> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<RuleExpression> {
        let mut left = self.parse_and()?;

        while matches!(self.current(), Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = RuleExpression::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<RuleExpression> {
        let mut left = self.parse_not()?;

        while matches!(self.current(), Token::And) {
            self.advance();
            let right = self.parse_not()?;
            left = RuleExpression::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_not(&mut self) -> Result<RuleExpression> {
        if matches!(self.current(), Token::Not) {
            self.advance();
            let expr = self.parse_not()?;
            return Ok(RuleExpression::Not(Box::new(expr)));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<RuleExpression> {
        match self.current().clone() {
            Token::LParen => {
                self.advance();
                let expr = self.parse_or()?;
                if !matches!(self.current(), Token::RParen) {
                    return Err(anyhow!("Expected ')' in expression"));
                }
                self.advance();
                Ok(expr)
            }
            Token::Atomic(expr) => {
                self.advance();
                Ok(expr)
            }
            Token::Eof => Err(anyhow!("Unexpected end of expression")),
            _ => Err(anyhow!("Unexpected token in expression")),
        }
    }
}

fn parse_atomic_expression(s: &str) -> Result<RuleExpression> {
    let s = s.trim();
    let s_lower = s.to_lowercase();

    // Generic: <field> matches "pattern" (special case for name)
    if let Some(match_pos) = s_lower.find(" matches ") {
        let field_name = &s[..match_pos];
        if field_name.to_lowercase() == "name" {
            let (value, _) = extract_quoted_string(&s[match_pos + 9..])?;
            let regex = Regex::new(&value).map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
            return Ok(RuleExpression::NameMatches(regex));
        } else {
            // Generic field matches
            let is_valid_field = matches!(
                field_name.to_lowercase().as_str(),
                "platform" | "language" | "team" | "role"
            );
            if is_valid_field {
                let (value, _) = extract_quoted_string(&s[match_pos + 9..])?;
                let regex =
                    Regex::new(&value).map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
                return Ok(RuleExpression::FieldMatches(
                    field_name.to_lowercase(),
                    regex,
                ));
            }
        }
    }

    // Generic: <field> in [value1, value2, ...]
    if let Some(bracket_pos) = s_lower.find(" in [") {
        let field_name = &s[..bracket_pos];
        let is_valid_field = matches!(
            field_name.to_lowercase().as_str(),
            "name" | "platform" | "language" | "team" | "role"
        );
        if is_valid_field {
            let list_start = bracket_pos + 5; // " in [".len()
            let (values, _) = extract_list_values(&s[list_start..])?;
            return Ok(RuleExpression::FieldIn(field_name.to_lowercase(), values));
        }
    }

    // Generic field existence: <field> exists
    if s_lower.ends_with(" exists") && s.len() > 7 {
        let field_name = &s[..s.len() - 7].trim();
        let is_valid_field = matches!(
            field_name.to_lowercase().as_str(),
            "team" | "platform" | "language" | "url" | "docs" | "ci" | "oncall" | "role"
        );
        if is_valid_field {
            return Ok(RuleExpression::FieldExists(field_name.to_lowercase()));
        }
    }

    Err(anyhow!("Unexpected expression: {}", s))
}

fn extract_quoted_string(s: &str) -> Result<(String, &str)> {
    let s = s.trim_start();
    let first_char = s
        .chars()
        .next()
        .ok_or_else(|| anyhow!("Expected string value"))?;

    // Check if it's a quoted string
    if first_char == '"' || first_char == '\'' {
        let rest = &s[1..];
        if let Some(pos) = rest.find(first_char) {
            return Ok((rest[..pos].to_string(), &rest[pos + 1..]));
        } else {
            return Err(anyhow!("Unclosed quoted string"));
        }
    }

    // Handle unquoted string - read until we hit a space, paren, or end of string
    let end_pos = s
        .find(|c: char| c.is_whitespace() || c == ')' || c == '(' || c == ',')
        .unwrap_or(s.len());

    Ok((s[..end_pos].to_string(), &s[end_pos..]))
}

fn extract_list_values(s: &str) -> Result<(Vec<String>, &str)> {
    if let Some(pos) = s.find(']') {
        let values: Vec<String> = s[..pos]
            .split(',')
            .map(|v| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
            .collect();
        Ok((values, &s[pos + 1..]))
    } else {
        Err(anyhow!("Unclosed list"))
    }
}

fn evaluate_expression(expr: &RuleExpression, service: &ServiceEntry) -> bool {
    match expr {
        RuleExpression::NameMatches(regex) => regex.is_match(&service.name),

        RuleExpression::FieldExists(field_name) => get_field_value(service, field_name).is_some(),

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

        RuleExpression::Contains(field_name, substring) => {
            if let Some(value) = get_field_value(service, field_name) {
                value.contains(substring)
            } else {
                false
            }
        }

        RuleExpression::NotEqual(field_name, expected) => {
            if let Some(value) = get_field_value(service, field_name) {
                value != *expected
            } else {
                true // Missing field is not equal to the value
            }
        }

        RuleExpression::Equal(field_name, expected) => {
            if let Some(value) = get_field_value(service, field_name) {
                value == *expected
            } else {
                false
            }
        }

        RuleExpression::And(left, right) => {
            evaluate_expression(left, service) && evaluate_expression(right, service)
        }

        RuleExpression::Or(left, right) => {
            evaluate_expression(left, service) || evaluate_expression(right, service)
        }

        RuleExpression::Not(expr) => !evaluate_expression(expr, service),
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
            base: None,
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
            base: None,
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
            base: None,
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
            base: None,
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
            base: None,
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
            base: None,
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
            base: None,
        };

        let result = RuleEngine::compile(&[rule]);
        assert!(result.is_err(), "Should reject invalid expression");
    }
}
