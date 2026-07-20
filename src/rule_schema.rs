//! Schema validation for inline policy rules (the `policy.rules` list in the
//! service manifest, see [`crate::manifest::PolicyConfig`] and
//! [`crate::rules::Rule`]).
//!
//! `svccat lint` calls [`validate`] so a structurally malformed policy rule
//! is caught with a clear, specific error *before* a user ever runs
//! `svccat check` / `workspace check`, where the same input previously
//! produced one of two bad outcomes instead of a lint error:
//!
//! - A rule that failed to compile (bad `severity`, unparsable `expression`,
//!   a dangling `base` reference) only ever surfaced as a non-fatal
//!   `eprintln!("Warning: Failed to compile custom rules: {e}")` in
//!   [`crate::drift::analyze`] — the command still exits success and every
//!   policy rule is silently disabled for that run.
//! - A rule whose `base` chain forms a cycle (including a rule naming
//!   itself as its own `base`) is not just silently ignored: compiling it
//!   recurses through [`crate::rules::RuleEngine::compile`]'s inheritance
//!   resolver with no cycle guard, so the process runs out of stack and
//!   crashes outright. Verified directly: a single rule with
//!   `base: Some(<own id>)` passed to `RuleEngine::compile` crashes the
//!   process with `STATUS_STACK_OVERFLOW` (0xc00000fd on Windows) rather
//!   than returning an `Err`.
//!
//! Because that second case means `RuleEngine::compile` itself is not safe
//! to call on unvalidated input, the checks here run in two phases: cheap
//! structural checks first (rule id uniqueness, non-blank ids, `base`
//! references that resolve, and — the one novel check with no existing
//! coverage anywhere — `base` chains that form a cycle), and only once the
//! rule set is confirmed safe to resolve does this module delegate to the
//! existing [`crate::rules::RuleEngine::compile`] for the semantic checks
//! (severity enum, expression syntax) it already performs, rather than
//! reimplementing that parser here.

use crate::lint::{LintIssue, LintSeverity};
use crate::rules::{Rule, RuleEngine};
use std::collections::{HashMap, HashSet};

/// Validate the structural shape of `rules`, then — only if that structure
/// is safe to resolve — delegate to [`RuleEngine::compile`] for the
/// semantic checks (severity enum, expression syntax) it already performs.
///
/// Returns one [`LintIssue`] per problem found. An empty result means the
/// rule set is safe to compile; a non-empty result always contains at least
/// one [`LintSeverity::Error`], matching the convention `lint::run` uses
/// elsewhere (see its blank-name and duplicate-name checks).
pub fn validate(rules: &[Rule]) -> Vec<LintIssue> {
    let mut issues = Vec::new();

    validate_blank_ids(rules, &mut issues);
    validate_unique_ids(rules, &mut issues);
    validate_base_references_exist(rules, &mut issues);
    validate_no_base_cycles(rules, &mut issues);

    // Any structural problem above makes the rule set unsafe (or at least
    // meaningless) to resolve: a cyclic or duplicate-id rule set is exactly
    // what RuleEngine::compile's inheritance resolver has no guard against,
    // and a dangling base reference makes its resolution result moot. Stop
    // here rather than risk running the same crash this module exists to
    // prevent.
    if !issues.is_empty() {
        return issues;
    }

    // Semantic validation (severity enum, expression syntax): reuse the
    // real compilation engine rather than re-implement its parser. This is
    // exactly what `svccat check` runs downstream, so lint now fails fast
    // on precisely what would otherwise only ever surface as a swallowed
    // warning there.
    if let Err(e) = RuleEngine::compile(rules) {
        issues.push(LintIssue {
            severity: LintSeverity::Error,
            message: format!("policy rule failed validation: {e}"),
        });
    }

    issues
}

fn validate_blank_ids(rules: &[Rule], issues: &mut Vec<LintIssue>) {
    for (i, rule) in rules.iter().enumerate() {
        if rule.id.trim().is_empty() {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: format!("policy rule at index {i} has a blank or whitespace-only id"),
            });
        }
    }
}

fn validate_unique_ids(rules: &[Rule], issues: &mut Vec<LintIssue>) {
    let mut indexes_by_id: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, rule) in rules.iter().enumerate() {
        indexes_by_id.entry(rule.id.as_str()).or_default().push(i);
    }

    let mut dupes: Vec<(&str, &Vec<usize>)> = indexes_by_id
        .iter()
        .filter(|(_, idxs)| idxs.len() > 1)
        .map(|(&id, idxs)| (id, idxs))
        .collect();
    dupes.sort_unstable_by_key(|(id, _)| *id);

    for (id, idxs) in dupes {
        let positions = idxs
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        issues.push(LintIssue {
            severity: LintSeverity::Error,
            message: format!(
                "policy rule id '{id}' is declared {} times (at indexes {positions}); \
                 rule ids must be unique because `base` inheritance and violation \
                 reports both key on id",
                idxs.len()
            ),
        });
    }
}

fn validate_base_references_exist(rules: &[Rule], issues: &mut Vec<LintIssue>) {
    let ids: HashSet<&str> = rules.iter().map(|r| r.id.as_str()).collect();
    for rule in rules {
        if let Some(base_id) = &rule.base {
            if !ids.contains(base_id.as_str()) {
                issues.push(LintIssue {
                    severity: LintSeverity::Error,
                    message: format!(
                        "policy rule '{}' has base '{base_id}', which does not match any rule id",
                        rule.id
                    ),
                });
            }
        }
    }
}

fn validate_no_base_cycles(rules: &[Rule], issues: &mut Vec<LintIssue>) {
    let by_id: HashMap<&str, &Rule> = rules.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut already_reported: HashSet<&str> = HashSet::new();

    for rule in rules {
        if already_reported.contains(rule.id.as_str()) {
            continue;
        }
        if let Some(cycle) = find_base_cycle(rule, &by_id) {
            for id in &cycle {
                already_reported.insert(id);
            }
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: format!(
                    "policy rule base chain forms a cycle: {}",
                    cycle.join(" -> ")
                ),
            });
        }
    }
}

/// Walk `rule`'s `base` chain looking for a repeated id (a self-reference is
/// a repeat after one hop). Returns the path from `rule` up to and including
/// the repeated id, or `None` if the chain terminates (or dangles — a
/// missing base is reported separately by `validate_base_references_exist`,
/// so it is treated here as "not a cycle" rather than re-diagnosed).
///
/// Iterative by construction: this is the guard that keeps
/// `RuleEngine::compile`'s own *recursive* resolver from ever being called
/// on a cyclic chain, which is what makes it crash instead of erroring.
fn find_base_cycle<'a>(rule: &'a Rule, by_id: &HashMap<&'a str, &'a Rule>) -> Option<Vec<&'a str>> {
    let mut path = vec![rule.id.as_str()];
    let mut current = rule;

    while let Some(base_id) = current.base.as_deref() {
        if path.contains(&base_id) {
            path.push(base_id);
            return Some(path);
        }
        match by_id.get(base_id) {
            Some(base_rule) => {
                path.push(base_id);
                current = base_rule;
            }
            None => return None,
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, severity: &str, expression: &str, base: Option<&str>) -> Rule {
        Rule {
            id: id.to_string(),
            description: format!("{id} description"),
            expression: expression.to_string(),
            severity: severity.to_string(),
            base: base.map(str::to_string),
        }
    }

    #[test]
    fn well_formed_rules_produce_no_issues() {
        let rules = vec![
            rule("required_team", "error", "team exists", None),
            rule(
                "critical_team",
                "error",
                "team exists",
                Some("required_team"),
            ),
        ];
        assert!(validate(&rules).is_empty());
    }

    #[test]
    fn blank_id_is_flagged() {
        let rules = vec![rule("", "error", "team exists", None)];
        let issues = validate(&rules);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, LintSeverity::Error);
        assert!(issues[0].message.contains("blank"));
    }

    #[test]
    fn duplicate_ids_are_flagged_by_name_and_index() {
        let rules = vec![
            rule("required_team", "error", "team exists", None),
            rule("required_team", "warning", "docs exists", None),
        ];
        let issues = validate(&rules);
        assert_eq!(
            issues.len(),
            1,
            "expected exactly one duplicate-id issue: {issues:?}"
        );
        assert!(issues[0].message.contains("required_team"));
        assert!(issues[0].message.contains("indexes 0, 1"));
    }

    #[test]
    fn dangling_base_reference_is_flagged() {
        let rules = vec![rule(
            "orphan",
            "error",
            "team exists",
            Some("does_not_exist"),
        )];
        let issues = validate(&rules);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("orphan"));
        assert!(issues[0].message.contains("does_not_exist"));
    }

    #[test]
    fn self_referencing_base_is_a_cycle() {
        let rules = vec![rule("self_ref", "error", "team exists", Some("self_ref"))];
        let issues = validate(&rules);
        assert_eq!(
            issues.len(),
            1,
            "expected exactly one cycle issue: {issues:?}"
        );
        assert!(issues[0].message.contains("cycle"));
        assert!(issues[0].message.contains("self_ref"));
    }

    #[test]
    fn mutual_two_rule_cycle_is_reported_once() {
        let rules = vec![
            rule("a", "error", "team exists", Some("b")),
            rule("b", "error", "team exists", Some("a")),
        ];
        let issues = validate(&rules);
        assert_eq!(
            issues.len(),
            1,
            "a two-node cycle should be reported once, not once per node: {issues:?}"
        );
        assert!(issues[0].message.contains("cycle"));
    }

    #[test]
    fn invalid_severity_is_surfaced_with_the_rule_id() {
        let rules = vec![rule("bad_severity", "critical", "team exists", None)];
        let issues = validate(&rules);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("bad_severity"));
        assert!(issues[0].message.contains("critical"));
    }

    #[test]
    fn unparsable_expression_is_surfaced_with_the_rule_id() {
        let rules = vec![rule(
            "bad_expression",
            "error",
            "this is not valid syntax",
            None,
        )];
        let issues = validate(&rules);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("bad_expression"));
    }

    #[test]
    fn structural_errors_short_circuit_before_reaching_compile() {
        // Two problems at once: a cycle AND a bad severity on the same
        // rule. If validate() ever called RuleEngine::compile on a cyclic
        // rule set, this input is exactly what would crash the process
        // (see the module doc comment). Confirms only the structural
        // (cycle) issue is reported, proving compile was never reached.
        let rules = vec![rule(
            "self_ref",
            "not-a-real-severity",
            "team exists",
            Some("self_ref"),
        )];
        let issues = validate(&rules);
        assert_eq!(issues.len(), 1, "expected only the cycle issue: {issues:?}");
        assert!(issues[0].message.contains("cycle"));
    }
}
