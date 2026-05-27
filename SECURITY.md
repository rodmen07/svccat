# Security Policy

svccat is a service catalog drift detection tool designed for use in development, CI/CD, and local automation contexts. This document outlines the security model, known limitations, and mitigations.

## Threat Model

svccat processes **untrusted manifest files** (`services.yaml`, `svccat.toml`) and **untrusted git history** (via `--since`) in a local repository context. The primary threat is a compromised or malicious repository introducing drift analysis to the development machine or CI runner.

### Attack Vectors Considered

- **Local attacker with repo commit access** can introduce malicious manifests or git refs
- **Compromised CI/CD pipeline** where untrusted code modifies the repository
- **Supply chain attacks** where dependencies are compromised
- **Configuration mistakes** exposing internal services to public scanning

### Out of Scope

- **Remote code execution via svccat itself** (svccat is not a web service or privileged daemon)
- **Privilege escalation** (svccat runs with user permissions)
- **Physical security** or OS-level attacks

---

## Security Mitigations

### 1. Git Command Injection (CRITICAL) ✅

**Vulnerability:** The `--since` flag accepted arbitrary git references that could trigger unexpected behavior.

**Mitigations:**
- ✅ Strict validation of git refs against allowlist pattern: `^[a-zA-Z0-9._/\-:@^~!+]+$`
- ✅ Rejection of empty refs, refs > 256 bytes, and patterns containing `..` or null bytes
- ✅ Use of `git show --` separator to prevent git from misinterpreting arguments
- ✅ Manifest path validation to prevent directory traversal in git specs
- ✅ Unit tests covering valid/invalid refs and path traversal attempts

**Affected Flag:** `svccat check --since <ref>`, `svccat report --history`

**Recommendation:** Always use version control tags or well-known branches for the `--since` flag. Avoid user-supplied git refs in CI/CD without validation.

---

### 2. SSRF in Service Ping & Webhooks (HIGH) ✅

**Vulnerability:** Service URLs from the manifest were not validated before making HTTP requests, allowing probing of internal infrastructure.

**Mitigations:**
- ✅ URL validation module (`urlvalidation.rs`) that rejects private/internal IPs
- ✅ Blocks IPv4 private ranges: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `127.0.0.0/8`, `169.254.0.0/16`
- ✅ Blocks IPv6 private ranges: loopback `::1`, private `fc00::/7`, link-local `fe80::/10`
- ✅ Enforces `https://` for webhooks (allows `http://localhost` for development)
- ✅ Added `PingStatus::Invalid` to indicate URL validation failures
- ✅ All output formats (terminal, markdown, junit) handle invalid URLs gracefully

**Affected Features:** `svccat check --ping`, webhook configuration in `svccat.toml`

**Recommendation:**
- Only declare public-facing service URLs in the manifest
- Use `localhost` URLs only in development and CI scripts, not in checked-in manifests
- Enable webhook validation (enabled by default) to prevent accidental data exfiltration

---

### 3. Deserialization Bombs (YAML/TOML) (HIGH) ✅

**Vulnerability:** YAML/TOML files with exponential expansion (anchor bomb), deep nesting, or large collections could exhaust memory and CPU.

**Mitigations:**
- ✅ Manifest files limited to **10 MB** (enforced before deserialization)
- ✅ Maximum **10,000 services** per manifest (sanity check)
- ✅ Service names limited to **256 bytes**
- ✅ `depends_on` lists limited to **1,000 entries** (prevents quadratic dependencies)
- ✅ Config files (`svccat.toml`) limited to **1 MB**
- ✅ Early rejection with clear error messages
- ✅ Graceful handling in config loading (logs warning, uses defaults)

**Affected Functions:** `Manifest::load()`, `webhook::load_config()`, and policy loading

**Recommendation:**
- Keep manifest files under 1 MB in practice (even with limits, large files are slow)
- Use tool-generated manifests (`svccat init`) rather than hand-crafted YAML
- Monitor manifest file size in CI: `ls -lh services.yaml`

---

### 4. Symlink Attacks in Discovery (HIGH) ⚠️ Pending

**Vulnerability:** The discovery algorithm could follow symlinks to files outside the repository, leaking information or causing infinite loops.

**Status:** Identified but not yet mitigated in this release. Scheduled for v0.19.

**Temporary Mitigations:**
- Run svccat in isolated CI environments with restricted filesystem access
- Avoid checking in symlinks in repository discovery paths
- Use `--ignore` to exclude directories with symlinks

---

### 5. Path Traversal in Manifest (HIGH) ⚠️ Pending

**Vulnerability:** The `path` field in service entries wasn't validated, potentially allowing `../../etc/passwd` or similar escapes.

**Status:** Identified but not yet mitigated. Scheduled for v0.19.

**Temporary Mitigations:**
- Generate manifest via `svccat init` rather than hand-editing
- Code review manifest changes: grep for `../` in `path` fields
- Run svccat on a clean checkout in CI (not user-modified directories)

---

### 6. Glob Pattern DoS (MEDIUM) ⚠️ Pending

**Vulnerability:** Unbounded glob patterns in `discovery.paths` could match millions of files, causing slowness or hangs.

**Status:** Identified. Partial mitigation via resource limits scheduled for v0.19.

**Temporary Mitigations:**
- Whitelist discovery paths: `services/*`, `microservices/*`, `apps/*`
- Avoid wildcard paths like `**/*` or `src/**/*/`
- Monitor discovery performance: add a `--discovery-timeout` locally if needed

---

### 7. Information Disclosure in Errors (MEDIUM)

**Vulnerability:** Error messages could leak system paths or git internals.

**Status:** Partially mitigated. Error messages redact sensitive details.

**Current Behavior:**
- File paths are displayed relative to repo root when possible
- Git command failures show stderr (may include system info)
- YAML parse errors show line numbers but not full context

**Recommendation:**
- When sharing svccat output in logs, review error messages for sensitive paths
- Avoid piping svccat output containing errors to untrusted systems

---

### 8. PowerShell Notification Script (MEDIUM) ⚠️

**Vulnerability:** The `svccat watch --notify` feature on Windows uses PowerShell with unescaped service names.

**Status:** Escape logic present (`'` → `''`) but not independently audited.

**Current Behavior:**
- Service names are escaped for PowerShell single-quoted strings
- Notification payload is generated dynamically

**Recommendation:**
- Avoid using `--notify` in untrusted environments
- If enabled, review service names in `services.yaml` to avoid special characters

---

## Dependency Security

### Current Dependencies

All direct dependencies are actively maintained and do not have known critical CVEs (as of svccat v0.18.0):

- `anyhow` 1.x — error handling
- `clap` 4.x — CLI argument parsing
- `serde` 1.x — serialization
- `serde_yaml` 0.9 — YAML parsing (no unsafe code)
- `ureq` 2.x — HTTP client
- `notify` 6.x — filesystem watching
- `toml` 0.8 — TOML parsing
- `glob` 0.3 — glob matching
- `colored` 2.x — terminal colors
- `url` 2.x — URL parsing (added for SSRF prevention)

### Vulnerability Scanning

**In CI:** Run `cargo audit` on every commit to detect transitive dependency vulnerabilities.

**Locally:** `cargo audit` or `cargo outdated` to check for updates.

**Recommendation:** Keep dependencies updated (especially `serde_yaml` and `url` for security patches).

---

## Usage Recommendations

### Safe Defaults

✅ **Recommended Configuration:**
```yaml
version: "1"
discovery:
  paths:
    - services/*
    - microservices/*
  markers:
    - Cargo.toml
    - go.mod
    - package.json
services:
  - name: api-service
    language: Rust
    platform: "cloud-run"  # Use well-known platform names only
    url: "https://api.example.com"  # Public endpoints only
    role: API gateway
    team: platform
```

### Unsafe Patterns to Avoid

❌ **Do NOT do this:**
```yaml
# ❌ URL points to internal IP
- name: internal-service
  url: "http://192.168.1.1:8000"

# ❌ Path with parent directory traversal
- name: config-service
  path: "../../config"

# ❌ Oversized manifest (>1MB uncompressed)
# ❌ Unrealistic number of services (>1000)

# ❌ Unknown platforms with special characters
platform: "my-custom/@#$-platform"
```

### In CI/CD Environments

1. **Isolate git operations:** Run svccat on clean checkouts
2. **Limit git refs:** Use tags or `origin/main` for `--since`, not user input
3. **Validate manifests:** Code review manifest changes before merging
4. **Monitor size:** `wc -c services.yaml` < 100KB in practice
5. **Enable audit:** `cargo audit` in CI to catch dependency issues

### Local Development

1. Use `svccat init` to scaffold manifests (safer than hand-editing YAML)
2. Avoid hand-writing complex YAML; use generated manifests
3. Test manifest changes: `svccat lint && svccat check`
4. Review diffs: `git diff services.yaml` before committing

---

## Reporting Security Issues

**Do NOT open public GitHub issues for security vulnerabilities.**

If you discover a security issue in svccat:

1. **Do not disclose publicly.** Do not open a GitHub issue, post on social media, or discuss in public channels.
2. **Email the maintainer** at [security@example.com](mailto:security@example.com) with:
   - Description of the vulnerability
   - Steps to reproduce (if applicable)
   - Suggested mitigation
3. **Expected timeline:** Maintainer will acknowledge within 48 hours and work toward a fix.

### Responsible Disclosure

- Allow at least **30 days** for a fix before public disclosure
- Credit the researcher in the security advisory and changelog
- Coordinate release timing with the researcher

---

## Known Limitations

### Not In Scope for svccat

- **Code execution in discovered services:** svccat does not execute service code
- **Vulnerability scanning of services:** svccat does not scan code for CVEs
- **Privilege escalation:** svccat runs with user permissions only
- **Web API security:** svccat is a CLI tool, not a web service

### Audit Trail

svccat **does not** produce audit logs. For compliance/audit requirements:
- Use `git log` to track manifest changes
- Use CI/CD platform audit logs for job execution history
- Integrate svccat output into your existing logging system

---

## Security Best Practices

### For Maintainers of Large Monorepos

1. **Version control the manifest:** Track `services.yaml` in git with signed commits
2. **Require code review:** Enforce approval before manifest changes
3. **Automate validation:** Run `svccat lint && svccat check --fail-on-drift` in CI
4. **Document ownership:** Use `team:` and `oncall:` fields for accountability
5. **Regular audits:** Run `svccat audit --cost-estimate` monthly to review catalog health

### For CI/CD Platform Engineers

1. **Isolate svccat runs:** Use dedicated CI jobs with minimal permissions
2. **Restrict manifest edit:** Only allow maintainers to modify `services.yaml`
3. **Cache discoveries:** Cache `svccat export` output to avoid repeated file walks on huge repos
4. **Monitor resource use:** Set timeouts on discovery (e.g., `--depth 1` for large repos)
5. **Integrate with observability:** Post svccat audit results to dashboards

---

## Changelog of Security Fixes

### v0.19.0 (Planned)

- [ ] Fix symlink attacks in discovery (detect symlinks, add `--follow-symlinks` flag)
- [ ] Fix path traversal in manifest `path` fields (normalize and validate)
- [ ] Add glob pattern DoS mitigations (depth limits, pattern whitelisting)
- [ ] Improve error message redaction (no full paths in output)
- [ ] Add security.md documentation

### v0.18.1 (Current)

- ✅ Git command injection prevention (git ref validation, manifest path validation)
- ✅ SSRF prevention (URL validation, private IP blocking)
- ✅ Deserialization bomb prevention (file size + resource limits)
- ✅ Added `url` crate for strict URL parsing

### Earlier Versions

- No security-specific changes in v0.18.0 and earlier

---

## Contact

For questions or security concerns:

- **GitHub Issues:** General questions and feature requests
- **Security Email:** [security@example.com](mailto:security@example.com) (private disclosure only)
- **Maintainer:** [@rodmen07](https://github.com/rodmen07)

---

## License

This security policy is part of svccat, released under the MIT License.

Last Updated: 2026-05-27
