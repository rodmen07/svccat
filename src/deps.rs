use crate::manifest::Manifest;
use anyhow::Result;
use colored::Colorize;
use std::collections::{HashMap, HashSet, VecDeque};

// ── Types ─────────────────────────────────────────────────────────────────────

/// An edge in the dependency graph.
#[derive(Debug, Clone)]
pub struct DepEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Default)]
pub struct DepsReport {
    pub edges: Vec<DepEdge>,
    /// `depends_on` targets that don't exist in the manifest.
    pub missing: Vec<DepEdge>,
    /// Detected cycles (each entry is the cycle path, e.g. "a -> b -> a").
    pub cycles: Vec<String>,
    pub total_services: usize,
    pub services_with_deps: usize,
}

impl DepsReport {
    pub fn has_errors(&self) -> bool {
        !self.missing.is_empty() || !self.cycles.is_empty()
    }
}

// ── Analysis ──────────────────────────────────────────────────────────────────

/// Build a dependency report from the manifest.
pub fn analyze(manifest: &Manifest) -> DepsReport {
    let known: HashSet<&str> = manifest.services.iter().map(|s| s.name.as_str()).collect();

    let mut edges = Vec::new();
    let mut missing = Vec::new();
    let mut services_with_deps = 0usize;

    for svc in &manifest.services {
        if svc.depends_on.is_empty() {
            continue;
        }
        services_with_deps += 1;
        for dep in &svc.depends_on {
            let edge = DepEdge {
                from: svc.name.clone(),
                to: dep.clone(),
            };
            if known.contains(dep.as_str()) {
                edges.push(edge);
            } else {
                missing.push(edge);
            }
        }
    }

    let cycles = detect_cycles(manifest);

    DepsReport {
        edges,
        missing,
        cycles,
        total_services: manifest.services.len(),
        services_with_deps,
    }
}

/// Detect cycles using iterative DFS (Kahn's topological sort).
fn detect_cycles(manifest: &Manifest) -> Vec<String> {
    let names: Vec<&str> = manifest.services.iter().map(|s| s.name.as_str()).collect();
    let index: HashMap<&str, usize> = names.iter().enumerate().map(|(i, n)| (*n, i)).collect();

    let n = names.len();
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    let mut in_degree: Vec<usize> = vec![0; n];

    for svc in &manifest.services {
        if let Some(&from_idx) = index.get(svc.name.as_str()) {
            for dep in &svc.depends_on {
                if let Some(&to_idx) = index.get(dep.as_str()) {
                    adj[from_idx].push(to_idx);
                    in_degree[to_idx] += 1;
                }
            }
        }
    }

    // Kahn's algorithm - nodes remaining after topological sort are in cycles.
    let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut processed = 0;

    while let Some(u) = queue.pop_front() {
        processed += 1;
        for &v in &adj[u] {
            in_degree[v] -= 1;
            if in_degree[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    if processed == n {
        return vec![];
    }

    // Find a representative cycle path for each remaining node using DFS.
    let cyclic: Vec<usize> = (0..n).filter(|&i| in_degree[i] > 0).collect();
    let cyclic_set: HashSet<usize> = cyclic.iter().cloned().collect();

    let mut cycles = Vec::new();
    let mut visited: HashSet<usize> = HashSet::new();

    for &start in &cyclic {
        if visited.contains(&start) {
            continue;
        }
        if let Some(path) = find_cycle_path(start, &adj, &cyclic_set, &names) {
            visited.insert(start);
            cycles.push(path);
        }
    }

    cycles
}

fn find_cycle_path(
    start: usize,
    adj: &[Vec<usize>],
    cyclic: &HashSet<usize>,
    names: &[&str],
) -> Option<String> {
    let mut stack: Vec<(usize, Vec<usize>)> = vec![(start, vec![start])];
    let mut visited: HashSet<usize> = HashSet::new();
    visited.insert(start);

    while let Some((node, path)) = stack.pop() {
        for &next in &adj[node] {
            if !cyclic.contains(&next) {
                continue;
            }
            if next == start {
                let mut full = path.clone();
                full.push(start);
                let s = full.iter().map(|&i| names[i]).collect::<Vec<_>>().join(" -> ");
                return Some(s);
            }
            if !visited.contains(&next) {
                visited.insert(next);
                let mut new_path = path.clone();
                new_path.push(next);
                stack.push((next, new_path));
            }
        }
    }
    None
}

// ── Renderers ─────────────────────────────────────────────────────────────────

/// Coloured terminal dependency summary.
pub fn render_terminal(report: &DepsReport) {
    println!("{}", "Dependency graph".bold().underline());
    println!(
        "  {} services total, {} have dependencies ({} edges)",
        report.total_services,
        report.services_with_deps,
        report.edges.len()
    );
    println!();

    if !report.edges.is_empty() {
        println!("{}", "Declared dependencies:".bold());
        for edge in &report.edges {
            println!("  {} -> {}", edge.from.cyan(), edge.to.cyan());
        }
        println!();
    }

    if !report.missing.is_empty() {
        println!("{}", "Undeclared targets (missing from manifest):".red().bold());
        for edge in &report.missing {
            println!("  {} -> {} (not found)", edge.from.yellow(), edge.to.red());
        }
        println!();
    }

    if !report.cycles.is_empty() {
        println!("{}", "Circular dependencies detected:".red().bold());
        for cycle in &report.cycles {
            println!("  {}", cycle.red());
        }
        println!();
    }

    if report.has_errors() {
        println!("{}", "Errors found. Fix the issues above.".red());
    } else {
        println!("{}", "No dependency errors.".green());
    }
}

/// Emit a Mermaid flowchart for the dependency graph.
pub fn render_mermaid(report: &DepsReport) {
    println!("```mermaid");
    println!("flowchart LR");
    for edge in &report.edges {
        println!("  {} --> {}", mermaid_id(&edge.from), mermaid_id(&edge.to));
    }
    for edge in &report.missing {
        println!(
            "  {}:::missing --> {}:::missing",
            mermaid_id(&edge.from),
            mermaid_id(&edge.to)
        );
    }
    if !report.missing.is_empty() {
        println!("  classDef missing fill:#f66,stroke:#c00,color:#fff");
    }
    println!("```");
}

pub fn render_json(report: &DepsReport) -> Result<()> {
    println!("{}", render_json_to_string(report)?);
    Ok(())
}

pub fn render_json_to_string(report: &DepsReport) -> Result<String> {
    let j = serde_json::json!({
        "total_services": report.total_services,
        "services_with_deps": report.services_with_deps,
        "edges": report.edges.iter().map(|e| serde_json::json!({"from": e.from, "to": e.to})).collect::<Vec<_>>(),
        "missing": report.missing.iter().map(|e| serde_json::json!({"from": e.from, "to": e.to})).collect::<Vec<_>>(),
        "cycles": report.cycles,
        "has_errors": report.has_errors(),
    });
    Ok(serde_json::to_string_pretty(&j)?)
}

fn mermaid_id(name: &str) -> String {
    let safe = name.replace([' ', '-', '.', '/'], "_");
    format!("{}[\"{}\"]", safe, name)
}
