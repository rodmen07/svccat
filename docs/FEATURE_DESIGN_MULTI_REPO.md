> **Status (2026-07-18): SUPERSEDED, historical design record only.** Multi-repo workspaces, cross-repo dependencies, and composable rules shipped in v0.21.0 on 2026-06-03. Current planning lives in [ROADMAP.md](../ROADMAP.md).

# Feature Design: Multi-Repo Support (v0.20.0 Candidate)

## Implementation slices

Recorded 2026-07-19. Phases 1-4 of this design (configuration and loading,
multi-repo drift detection, reporting with cross-repo dependency analysis, and the
CLI integration as `svccat workspace check`) shipped together in v0.21.0 on
2026-06-03. The slices below cover only the remainder that never shipped, each
sized for one small PR. ROADMAP.md stays the planning source of truth; slice 3 is
the `workspace check --format html` candidate already listed there.

- [x] **Slice 1: workspace config completion and repo filtering** (implemented
  2026-07-19, same commit as this slice list). Parse the `[workspace]` `name` and
  `description` fields from section 1, carry the workspace name into the
  aggregated report and all three output formats (section 6's `workspace_name`),
  and wire the previously parsed-but-ignored `workspace check --filter <repos>`
  flag into real repo selection with unknown-name validation. Repos with
  `enabled = false` stay skipped even when named in the filter.
- [ ] **Slice 2: `[reporting]` config section.** Config-driven defaults from
  section 1: default output `format`, an `include_cross_repo_deps` toggle for the
  dependency analysis, and `exclude_patterns` merged into discovery ignore globs.
- [ ] **Slice 3: `workspace check --format html` interactive visualization.**
  Aggregated workspace report as a self-contained HTML page reusing the existing
  D3 graph renderer; tracked as a "Later / candidates" item in ROADMAP.md.

**Priority:** High (addresses team-scale drift detection)  
**Effort Estimate:** 2-3 weeks  
**Breaking Change:** No (backward compatible)

## Overview

Enable svccat to detect service drift across multiple git repositories in a single run, with aggregated reporting and cross-repo dependency analysis.

## Current State (v0.19.0)

```bash
# Current workflow: Must run separately per repo
svccat check -m repo1/services.yaml
svccat check -m repo2/services.yaml
# Results must be manually aggregated
```

## Proposed Design

### 1. Configuration Format

**svccat.toml** (new root config):
```toml
[workspace]
name = "Platform Engineering"
description = "Multi-service platform"

[[repos]]
name = "api-services"
path = "."  # Relative to svccat.toml
manifest = "services.yaml"

[[repos]]
name = "infrastructure"
path = "../infra-repo"
manifest = "services.yaml"
enabled = true

[[repos]]
name = "legacy"
path = "../legacy-code"
manifest = "manifest.yaml"
enabled = false  # Skip in checks

[reporting]
format = "json"  # json, table, markdown
include_cross_repo_deps = true
exclude_patterns = []  # Globs to skip
```

### 2. File Structure

```
svccat.toml                  # Workspace root (new)
├── services.yaml           # Default repo manifest
├── repo1/
│   └── services.yaml
└── repo2/
    └── services.yaml
```

### 3. Core Changes

#### `src/workspace.rs` (NEW)
```rust
pub struct Workspace {
    pub name: String,
    pub repos: Vec<RepositoryConfig>,
}

pub struct RepositoryConfig {
    pub name: String,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
    pub enabled: bool,
}

impl Workspace {
    pub fn load(root: &Path) -> Result<Self>;
    pub fn load_all_repos(&self) -> Result<Vec<(String, Manifest)>>;
    pub fn check_all_drift(&self) -> Result<Vec<DriftReport>>;
}
```

#### `src/lib.rs` (MODIFIED)
```rust
pub mod workspace;  // New module
// Existing modules...
```

#### `src/main.rs` (MODIFIED)
```rust
// New command: check-workspace (or detect-workspace)
#[derive(Subcommand)]
enum Command {
    Check {
        #[arg(short, long)]
        manifest: PathBuf,
    },
    CheckWorkspace {  // NEW
        #[arg(long, default_value = "svccat.toml")]
        config: PathBuf,
    },
    // ...
}
```

### 4. Implementation Phases

#### Phase 1: Configuration & Loading
- [ ] Create `workspace.rs` module
- [ ] Implement `Workspace::load()` parsing svccat.toml
- [ ] Add toml dependency to Cargo.toml
- [ ] Tests for config parsing

#### Phase 2: Multi-Repo Drift Detection
- [ ] `Workspace::load_all_repos()` loads all repo manifests
- [ ] `Workspace::check_all_drift()` runs drift for each repo
- [ ] Aggregate results into `AggregatedReport`
- [ ] Tests for drift detection across repos

#### Phase 3: Reporting
- [ ] Cross-repo dependency analysis
- [ ] Report format options (JSON, table, markdown)
- [ ] Console output formatting
- [ ] Export to file

#### Phase 4: CLI Integration
- [ ] `svccat check-workspace` command
- [ ] Backward compatibility with existing `svccat check`
- [ ] Documentation and examples

### 5. Output Example

```bash
$ svccat check-workspace
Checking 3 repositories...

┌─ api-services ─────────────────────────────────────────────┐
│ ✅ Services:            12 (3 discovered)                  │
│ ✅ Manifest size:       2.4 KB                             │
│ 🔶 Drift detected:      2 services behind main             │
│   - auth-service (commit 3 days old)                       │
│   - payment-service (main)                                 │
└────────────────────────────────────────────────────────────┘

┌─ infrastructure ───────────────────────────────────────────┐
│ ✅ Services:            8 (1 discovered)                   │
│ ✅ Manifest size:       1.8 KB                             │
│ ✅ No drift detected                                       │
└────────────────────────────────────────────────────────────┘

┌─ legacy ───────────────────────────────────────────────────┐
│ ⏭️  Skipped (disabled in config)                           │
└────────────────────────────────────────────────────────────┘

Summary:
  Total repos:          3
  Checked:              2
  Drift found:          2 services
  Across-repo deps:     5
  
Recommendations:
  1. Update api-services/auth-service (3 days old)
  2. Review cross-repo dependency: api-services → infrastructure
```

### 6. Data Structure

```rust
pub struct AggregatedReport {
    pub workspace_name: String,
    pub generated_at: DateTime<Utc>,
    pub repos: Vec<RepositoryReport>,
    pub cross_repo_deps: Vec<CrossRepoDep>,
    pub summary: Summary,
}

pub struct RepositoryReport {
    pub name: String,
    pub path: PathBuf,
    pub drift: DriftReport,  // Existing struct
}

pub struct CrossRepoDep {
    pub from: (String, String),  // (repo, service)
    pub to: (String, String),    // (repo, service)
    pub dependency_type: String,  // "webhook", "import", etc
}

pub struct Summary {
    pub total_repos: usize,
    pub checked_repos: usize,
    pub services_with_drift: usize,
    pub cross_repo_dependencies: usize,
}
```

### 7. Backward Compatibility

```rust
// Existing `svccat check -m services.yaml` still works
// New `svccat check-workspace` for multi-repo
// Both commands supported simultaneously
```

### 8. Testing Strategy

**Unit Tests** (`tests/workspace_tests.rs`):
- Parse valid svccat.toml
- Parse invalid configs (missing repo, circular deps)
- Load repos with mixed manifest paths
- Drift detection per repo

**Integration Tests**:
- Multi-repo setup with test fixtures
- Cross-repo dependency detection
- Report generation

**Example Test Setup**:
```
tests/fixtures/multi-repo/
├── svccat.toml
├── api-services/
│   ├── services.yaml
│   └── .git/
├── infrastructure/
│   ├── manifest.yaml
│   └── .git/
└── legacy/  (disabled)
```

### 9. Documentation Updates

- [ ] README: Multi-repo workflow section
- [ ] Configuration guide with examples
- [ ] Migration guide (single repo → multi-repo)
- [ ] Best practices for monorepo structures
- [ ] FAQ: Dependency management across repos

### 10. Success Criteria

- ✅ Load and validate svccat.toml configs
- ✅ Drift detection works across 3+ repos
- ✅ Cross-repo dependencies detected
- ✅ All output formats working (JSON, table, markdown)
- ✅ Backward compatible with single-repo workflows
- ✅ 10+ new tests with 95%+ coverage
- ✅ Performance acceptable for 10+ repos

### 11. Known Challenges & Solutions

| Challenge | Solution |
|-----------|----------|
| Git auth across multiple repos | Inherit from host git config; document per-repo setup |
| Circular cross-repo deps | Detect and report; validate in CI |
| Performance (many repos) | Parallel git operations; cache manifest loads |
| Path resolution | Make all paths relative to svccat.toml parent |

### 12. Future Extensions

- Workspace-level policies (apply rules to all repos)
- Workspace graph visualization
- Scheduled workspace checks
- Workspace health dashboard
- Integration with dependency management tools (Renovate, etc)

---

**Status:** Design complete, awaiting community feedback  
**Next Step:** Prioritize based on v0.19.0 user feedback
