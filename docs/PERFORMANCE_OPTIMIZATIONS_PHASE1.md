> **Status (2026-07-18): DELIVERED, historical record only.** Phase 1 work was completed 2026-05-30 and released in v0.20.0 on 2026-06-01, and the Phase 2 follow-up (rayon parallel discovery) shipped in v1.4.0 on 2026-07-09. Current planning lives in [ROADMAP.md](../ROADMAP.md).

# Phase 1: Performance Optimizations - v0.20.0

**Completion Date:** 2026-05-30  
**Goal:** Establish performance foundation before multi-repo support (Feature #1)

## Optimizations Implemented

### 1. Drift Detection Loop Optimization (drift.rs)

**Problem:** Lookup of declared services by path and name used O(n) operations.

```rust
// Before (O(n) for each discovered service)
let declared_paths: Vec<&str> = manifest.services.iter()...collect();
let declared_names: Vec<&str> = manifest.services.iter()...collect();

for disc in discovered {
    let matched_by_path = declared_paths.iter().any(|p| *p == disc.path);  // O(n)
    let matched_by_name = declared_names.contains(&disc.name.as_str());    // O(n)
}
```

**Solution:** Use `HashSet<&str>` for O(1) lookups.

```rust
// After (O(1) for each discovered service)
let declared_paths: std::collections::HashSet<&str> = manifest.services.iter()...collect();
let declared_names: std::collections::HashSet<&str> = manifest.services.iter()...collect();

for disc in discovered {
    let matched_by_path = declared_paths.contains(disc.path.as_str());    // O(1)
    let matched_by_name = declared_names.contains(disc.name.as_str());    // O(1)
}
```

**Impact:** For a manifest with 1000 services and 500 discovered services, this optimization reduces O(n*m) to O(n+m) lookups.

**File:** `src/drift.rs` lines 152-162

---

### 2. Discovery Deduplication Optimization (discovery.rs)

**Problem:** Path deduplication used O(n) `.any()` lookup for each newly discovered service.

```rust
// Before (O(n) deduplication check)
let mut discovered: Vec<DiscoveredService> = Vec::new();

for pattern in &patterns {
    // ... iterate entries ...
    if discovered.iter().any(|d| d.path == rel_path) {  // O(n)
        continue;
    }
    discovered.push(...);
}
```

**Solution:** Use `HashSet` to track seen paths for O(1) lookups.

```rust
// After (O(1) deduplication check)
let mut discovered: Vec<DiscoveredService> = Vec::new();
let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

for pattern in &patterns {
    // ... iterate entries ...
    if !seen_paths.insert(rel_path.clone()) {  // O(1)
        continue;
    }
    discovered.push(...);
}
```

**Impact:** For repos with overlapping glob patterns, reduces duplicate detection from O(n) to O(1) per discovery.

**File:** `src/discovery.rs` lines 116-182

---

### 3. Large-Manifest Benchmarks (benches/)

**Addition:** Created `large_manifest_benchmark.rs` for stress-testing performance with scaled-up manifests.

**Benchmark Scenarios:**
- Manifest parsing: 100, 500, 1000, 5000 services
- Discovery filtering: 100, 500, 1000 services
- Service lookup performance

**Purpose:** Establish baseline metrics and detect regressions in future changes.

**File:** `benches/large_manifest_benchmark.rs`

**Build Configuration:** Added to `Cargo.toml`

```toml
[[bench]]
name = "large_manifest_benchmark"
harness = false
```

---

## Files Modified

| File | Changes | Impact |
|------|---------|--------|
| `src/drift.rs` | Changed `Vec<&str>` to `HashSet<&str>` for name/path lookups | O(n) → O(1) lookups in hot path |
| `src/discovery.rs` | Added `HashSet<String>` for path deduplication | O(n) → O(1) deduplication |
| `benches/large_manifest_benchmark.rs` | Created new benchmark suite | Scalability testing (100-5000 services) |
| `Cargo.toml` | Added benchmark target | Build configuration |

---

## Expected Performance Impact

### Drift Detection (Hot Path)

**Scenario:** Manifest with 1000 services, 500 discovered services

**Before:**
- Declared names lookup: 500 × 1000 = 500,000 operations
- Declared paths lookup: 500 × 1000 = 500,000 operations
- **Total:** 1,000,000 operations

**After:**
- Declared names lookup: 500 × 1 = 500 operations
- Declared paths lookup: 500 × 1 = 500 operations
- **Total:** 1,000 operations

**Expected Speedup:** 1000x for the lookup phase (overhead from HashMap operations keeps real-world improvement to ~5-10x depending on service count)

---

### Discovery Deduplication

**Scenario:** 20 discovery patterns with overlap, 100 discovered services

**Before:**
- Per-service dedup check: 100 × (1 + 2 + 3 + ... + 100) = 505,050 operations

**After:**
- Per-service dedup check: 100 × 1 = 100 operations

**Expected Speedup:** 5000x for deduplication (practically ~100x with overhead)

---

## Testing & Verification

### Run benchmarks:

```bash
# Run large-manifest benchmarks
cargo bench --bench large_manifest_benchmark

# Run existing benchmarks
cargo bench --bench benchmark

# Run all tests
cargo test

# Verify no regressions
cargo clippy -- -D warnings
```

### Success Criteria:

- ✅ Large manifest parsing (1000+ services) completes in < 2 seconds
- ✅ No performance regressions in existing benchmarks
- ✅ Code compiles with `cargo clippy -- -D warnings`
- ✅ All tests pass

---

## Next Steps (Phase 2)

These optimizations establish the foundation for:

1. **Multi-Repo Support**: With 5-10 repos, the optimized loops prevent 5-10x slowdown
2. **Custom Validation Rules**: Efficient policy evaluation requires fast service lookups
3. **Advanced Reporting**: Traversing large manifests for analytics

---

## Technical Notes

### Why HashSet instead of HashMap?

For this use case, we only need membership testing (O(1)), not key-value storage. HashSet is simpler and has lower memory overhead than HashMap.

### Why not iterator optimization?

Iterator optimization patterns (consuming iterators, avoiding `.collect()`) were already in place. The bottleneck was algorithmic (O(n) vs O(1)), not iterator overhead.

### Impact on code clarity?

Changes are minimal and preserve code readability. HashSet usage is idiomatic Rust for "did I see this before?" operations.

---

**Phase Status:** ✅ COMPLETE  
**Performance Baseline:** Established (benches/large_manifest_benchmark.rs)  
**Ready for Phase 2:** Multi-Repo Support implementation
