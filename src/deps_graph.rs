use crate::manifest::Manifest;
use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize)]
pub struct ServiceKey {
    pub repo: String,
    pub service: String,
}

impl fmt::Display for ServiceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.repo, self.service)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphNode {
    pub key: ServiceKey,
    pub version: Option<String>,
    pub dependencies: Vec<DependencyEdge>,
    pub dependents: Vec<ServiceKey>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyEdge {
    pub target: ServiceKey,
    pub version_constraint: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<ServiceKey, GraphNode>,
    pub circular_dependencies: Vec<CircularDependency>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CircularDependency {
    pub cycle: Vec<ServiceKey>,
    pub description: String,
}

impl DependencyGraph {
    /// Build a dependency graph from all workspace repositories and their manifests
    pub fn build(repos: Vec<(String, &Manifest)>) -> Result<Self> {
        let mut nodes = HashMap::new();

        // Phase 1: Create nodes for all services in all repos
        for (repo_name, manifest) in &repos {
            for service in &manifest.services {
                let key = ServiceKey {
                    repo: repo_name.clone(),
                    service: service.name.clone(),
                };

                let node = GraphNode {
                    key: key.clone(),
                    version: None, // Version information can be added to ServiceEntry in future
                    dependencies: Vec::new(),
                    dependents: Vec::new(),
                };

                nodes.insert(key, node);
            }
        }

        // Phase 2: Wire up dependencies from depends_on references
        for (repo_name, manifest) in &repos {
            for service in &manifest.services {
                let source_key = ServiceKey {
                    repo: repo_name.clone(),
                    service: service.name.clone(),
                };

                for dep in &service.depends_on {
                    // Parse dependency reference (could be "service" or "repo:service")
                    let target_key = if dep.contains(':') {
                        let parts: Vec<&str> = dep.split(':').collect();
                        if parts.len() == 2 {
                            ServiceKey {
                                repo: parts[0].to_string(),
                                service: parts[1].to_string(),
                            }
                        } else {
                            ServiceKey {
                                repo: repo_name.clone(),
                                service: dep.clone(),
                            }
                        }
                    } else {
                        ServiceKey {
                            repo: repo_name.clone(),
                            service: dep.clone(),
                        }
                    };

                    // Add dependency edge
                    if let Some(source_node) = nodes.get_mut(&source_key) {
                        source_node.dependencies.push(DependencyEdge {
                            target: target_key.clone(),
                            version_constraint: None,
                        });
                    }

                    // Add reverse link (dependent)
                    if let Some(target_node) = nodes.get_mut(&target_key) {
                        target_node.dependents.push(source_key.clone());
                    }
                }
            }
        }

        // Phase 3: Detect circular dependencies
        let circular_dependencies = Self::detect_cycles_in_graph(&nodes)?;

        Ok(DependencyGraph {
            nodes,
            circular_dependencies,
        })
    }

    /// Detect circular dependencies using DFS
    fn detect_cycles_in_graph(
        nodes: &HashMap<ServiceKey, GraphNode>,
    ) -> Result<Vec<CircularDependency>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();

        for node_key in nodes.keys() {
            if !visited.contains(node_key) {
                Self::dfs_detect_cycles(
                    node_key,
                    nodes,
                    &mut visited,
                    &mut rec_stack,
                    Vec::new(),
                    &mut cycles,
                )?;
            }
        }

        Ok(cycles)
    }

    /// DFS helper for cycle detection
    fn dfs_detect_cycles(
        node_key: &ServiceKey,
        nodes: &HashMap<ServiceKey, GraphNode>,
        visited: &mut HashSet<ServiceKey>,
        rec_stack: &mut HashSet<ServiceKey>,
        mut path: Vec<ServiceKey>,
        cycles: &mut Vec<CircularDependency>,
    ) -> Result<()> {
        visited.insert(node_key.clone());
        rec_stack.insert(node_key.clone());
        path.push(node_key.clone());

        if let Some(node) = nodes.get(node_key) {
            for dep_edge in &node.dependencies {
                if !visited.contains(&dep_edge.target) {
                    Self::dfs_detect_cycles(
                        &dep_edge.target,
                        nodes,
                        visited,
                        rec_stack,
                        path.clone(),
                        cycles,
                    )?;
                } else if rec_stack.contains(&dep_edge.target) {
                    // Found a cycle
                    if let Some(start_idx) = path.iter().position(|k| k == &dep_edge.target) {
                        let cycle = path[start_idx..].to_vec();
                        let description = format!(
                            "Circular dependency: {}",
                            cycle
                                .iter()
                                .map(|k| k.to_string())
                                .collect::<Vec<_>>()
                                .join(" → ")
                        );

                        cycles.push(CircularDependency { cycle, description });
                    }
                }
            }
        }

        rec_stack.remove(node_key);
        Ok(())
    }

    /// Find all services that depend on a given service
    pub fn find_dependents(&self, service_key: &ServiceKey) -> Vec<ServiceKey> {
        if let Some(node) = self.nodes.get(service_key) {
            node.dependents.clone()
        } else {
            Vec::new()
        }
    }

    /// Find all services that a given service depends on
    pub fn find_dependencies(&self, service_key: &ServiceKey) -> Vec<ServiceKey> {
        if let Some(node) = self.nodes.get(service_key) {
            node.dependencies.iter().map(|e| e.target.clone()).collect()
        } else {
            Vec::new()
        }
    }

    /// Get the impact of a service change (what would break)
    pub fn get_impact(&self, service_key: &ServiceKey) -> ImpactAnalysis {
        let mut directly_affected = Vec::new();
        let mut transitively_affected = HashSet::new();

        // BFS to find all affected services
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back(service_key.clone());
        visited.insert(service_key.clone());

        while let Some(current) = queue.pop_front() {
            for dependent in self.find_dependents(&current) {
                if !visited.contains(&dependent) {
                    visited.insert(dependent.clone());
                    directly_affected.push(dependent.clone());
                    queue.push_back(dependent);
                }
            }
        }

        // Categorize affected services
        for service in &directly_affected {
            if self.find_dependents(service).is_empty() {
                // Leaf node - directly affected
                transitively_affected.insert(service.clone());
            }
        }

        ImpactAnalysis {
            directly_affected: directly_affected.clone(),
            transitively_affected,
            total_affected: directly_affected.len(),
        }
    }

    /// Check if all dependencies are resolvable
    pub fn validate_all_dependencies(&self) -> Vec<UnresolvableDependency> {
        let mut unresolvable = Vec::new();

        for (service_key, node) in &self.nodes {
            for dep_edge in &node.dependencies {
                if !self.nodes.contains_key(&dep_edge.target) {
                    unresolvable.push(UnresolvableDependency {
                        service: service_key.clone(),
                        dependency: dep_edge.target.clone(),
                        reason: format!("Service {} not found in workspace", dep_edge.target),
                    });
                }
            }
        }

        unresolvable
    }

    /// Get a summary of the dependency graph
    pub fn summary(&self) -> DependencySummary {
        let mut services_with_deps = 0;
        let mut total_dependencies = 0;
        let mut cross_repo_dependencies = 0;

        for node in self.nodes.values() {
            if !node.dependencies.is_empty() {
                services_with_deps += 1;
            }

            for dep in &node.dependencies {
                total_dependencies += 1;
                if dep.target.repo != node.key.repo {
                    cross_repo_dependencies += 1;
                }
            }
        }

        DependencySummary {
            total_services: self.nodes.len(),
            services_with_dependencies: services_with_deps,
            total_dependencies,
            cross_repo_dependencies,
            circular_dependencies: self.circular_dependencies.len(),
            unresolvable_dependencies: self.validate_all_dependencies().len(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImpactAnalysis {
    pub directly_affected: Vec<ServiceKey>,
    pub transitively_affected: HashSet<ServiceKey>,
    pub total_affected: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UnresolvableDependency {
    pub service: ServiceKey,
    pub dependency: ServiceKey,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencySummary {
    pub total_services: usize,
    pub services_with_dependencies: usize,
    pub total_dependencies: usize,
    pub cross_repo_dependencies: usize,
    pub circular_dependencies: usize,
    pub unresolvable_dependencies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manifest(service_names: Vec<&str>, dependencies: Vec<(&str, &str)>) -> Manifest {
        let mut manifest = Manifest {
            version: "1".to_string(),
            discovery: crate::manifest::DiscoveryConfig {
                paths: vec![],
                markers: vec![],
                ignore: vec![],
            },
            policy: crate::manifest::PolicyConfig {
                require_fields: vec![],
                rules: vec![],
            },
            services: vec![],
        };

        for name in service_names {
            manifest.services.push(crate::manifest::ServiceEntry {
                name: name.to_string(),
                language: Some("Rust".to_string()),
                platform: Some("Cloud Run".to_string()),
                url: None,
                role: None,
                team: None,
                oncall: None,
                submodule: None,
                path: None,
                docs: None,
                ci: None,
                tags: Vec::new(),
                depends_on: Vec::new(),
            });
        }

        for (service, dep) in dependencies {
            if let Some(svc) = manifest.services.iter_mut().find(|s| s.name == service) {
                svc.depends_on.push(dep.to_string());
            }
        }

        manifest
    }

    #[test]
    fn test_build_simple_graph() {
        let manifest = create_test_manifest(
            vec!["api", "auth", "db"],
            vec![("api", "auth"), ("api", "db")],
        );

        let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)]).unwrap();

        assert_eq!(graph.nodes.len(), 3);
        let api_key = ServiceKey {
            repo: "backend".to_string(),
            service: "api".to_string(),
        };
        let api_node = &graph.nodes[&api_key];
        assert_eq!(api_node.dependencies.len(), 2);
    }

    #[test]
    fn test_detect_circular_dependency() {
        let manifest =
            create_test_manifest(vec!["api", "auth"], vec![("api", "auth"), ("auth", "api")]);

        let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)]).unwrap();

        assert!(!graph.circular_dependencies.is_empty());
    }

    #[test]
    fn test_find_dependents() {
        let manifest = create_test_manifest(
            vec!["api", "auth", "web"],
            vec![("api", "auth"), ("web", "auth")],
        );

        let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)]).unwrap();

        let auth_key = ServiceKey {
            repo: "backend".to_string(),
            service: "auth".to_string(),
        };

        let dependents = graph.find_dependents(&auth_key);
        assert_eq!(dependents.len(), 2);
    }

    #[test]
    fn test_get_impact() {
        let manifest = create_test_manifest(
            vec!["api", "auth", "web"],
            vec![("api", "auth"), ("web", "api")],
        );

        let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)]).unwrap();

        let auth_key = ServiceKey {
            repo: "backend".to_string(),
            service: "auth".to_string(),
        };

        let impact = graph.get_impact(&auth_key);
        assert!(impact.total_affected > 0);
    }

    #[test]
    fn test_cross_repo_dependencies() {
        let manifest1 = create_test_manifest(vec!["api"], vec![("api", "backend:auth")]);
        let manifest2 = create_test_manifest(vec!["auth"], vec![]);

        let graph = DependencyGraph::build(vec![
            ("frontend".to_string(), &manifest1),
            ("backend".to_string(), &manifest2),
        ])
        .unwrap();

        let api_key = ServiceKey {
            repo: "frontend".to_string(),
            service: "api".to_string(),
        };

        let deps = graph.find_dependencies(&api_key);
        assert!(!deps.is_empty());
        assert_eq!(deps[0].repo, "backend");
    }

    #[test]
    fn test_dependency_summary() {
        let manifest = create_test_manifest(
            vec!["api", "auth", "db"],
            vec![("api", "auth"), ("api", "db")],
        );

        let graph = DependencyGraph::build(vec![("backend".to_string(), &manifest)]).unwrap();
        let summary = graph.summary();

        assert_eq!(summary.total_services, 3);
        assert_eq!(summary.total_dependencies, 2);
        assert_eq!(summary.circular_dependencies, 0);
    }
}
