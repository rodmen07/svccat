# Release Plan: svccat v1.4.0

This plan outlines the objectives, candidate features, backward-compatibility considerations, and implementation milestones for the next minor release, **v1.4.0**.

---

## 📅 Release Timeline & Goals

- **Target Version:** `1.4.0` (Minor Release - Backward Compatible)
- **Primary Goal:** Expand ecosystem interoperability (focusing on Backstage and OpenAPI/Compose integration) and strengthen monorepo scale performance (Phase 2 Optimizations).

---

## 🚀 Key Feature Categories

### 1. Unified Backstage Export (`--format backstage`)

Currently, `svccat` supports importing from Backstage catalogs (`svccat import --from backstage`), but lacks a native export format to generate Backstage-conforming `catalog-info.yaml` manifests.

- **Objective:** Enable platform engineers to use `svccat` as the source-of-truth drift-checker and export to Backstage cleanly.
- **CLI Command change:**
  ```bash
  svccat export --format backstage-yaml --output catalog-info.yaml
  ```
- **Technical Scope:** Map `ServiceEntry` fields (`name`, `language`, `platform`, `url`, `role`, `team`, `oncall`, `tags`, `depends_on`) to standard Backstage `Component` spec schemas.

### 2. Multi-threaded Service Discovery (Phase 2 Performance)

Medium-to-large corporate monorepos with 1000+ potential service directories can feel discovery-latency inside GHA workflows when scanning deep nested hierarchies.

- **Objective:** Parallelize service scanning and glob checking during discovery.
- **Technical Scope:** 
  - Integrate the `rayon` crate for parallel iterator execution over discovery patterns in `src/discovery.rs`.
  - Aim to reduce latency of a 5,000-directory glob check by **up to 70%** on multi-core runner environments.

### 3. Native Schema Validation for Custom Rules

Custom policy rules (`policy.rules`) currently compile into a rule evaluation engine, but rules can feel verbose to debug.

- **Objective:** Add schema linting and execution dry-runs for team policy configs.
- **CLI Command change:** Enhance `svccat lint` to also validate `.svccat/policy.yaml` rule expressions and configuration syntax before pipeline execution.

### 4. Interactive Workspace Visualization (`svccat workspace view`)

Enable offline-first browser visualization of multi-repository dependency topologies.

- **Objective:** Expand workspace checks to export a single unified HTML view.
- **CLI Command change:**
  ```bash
  svccat workspace check --format html --output workspace.html
  ```

---

## 🔒 Security & Robustness Gates

- **SSRF Sanitization:** Verify that `--ping` checks correctly drop DNS re-binding and local network resolution during redirects.
- **Fuzzing targets:** Expand parsing fuzzers inside `fuzz/fuzz_targets/fuzz_manifest.rs` to cover custom rules with circular base-rules.

---

## 📋 Implementation Phases

### Phase 1: Performance & Core Upgrades (Milestone 1)
- [ ] Add `rayon` dependency block to `Cargo.toml`.
- [ ] Refactor `discovery.rs` loop paths to use `into_par_iter()`.
- [ ] Benchmark local target speeds on oversized tests.

### Phase 2: Integration & Ecosystem Formats (Milestone 2)
- [ ] Create `src/output/backstage.rs` serialization schema.
- [ ] Introduce Backstage exporter unit tests.
- [ ] Bind format parser to `ExportFormat::BackstageYaml` in `src/cli.rs`.

### Phase 3: Workspace & Verification (Milestone 3)
- [ ] Hook workspace check to unified D3.js compiler logic.
- [ ] Align GHA CI configurations to skip unnecessary parallel tool fetches.
- [ ] Execute release bump to `1.4.0` with verified tests.
