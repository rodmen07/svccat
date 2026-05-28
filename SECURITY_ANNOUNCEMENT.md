# 🔒 Security Release v0.19.0 - Announcement

**Released:** May 28, 2026  
**Crates.io:** https://crates.io/crates/svccat/0.19.0  
**GitHub:** https://github.com/rodmen07/svccat/releases/tag/v0.19.0

## Overview

svccat v0.19.0 is a **critical security release** addressing **10 vulnerabilities** across multiple attack vectors. All users should upgrade immediately.

### Severity Breakdown
- 🔴 **1 Critical** - Git command injection
- 🟠 **2 High** - SSRF, Deserialization bombs
- 🟡 **7 Medium** - Path traversal, symlinks, glob DoS, info disclosure, IPv6, cross-platform, dependency scanning

## Key Security Fixes

### 🔴 Critical: Git Command Injection
**Impact:** Code execution in compromised repositories
**Fix:** Strict git ref validation with allowlist pattern
**Details:** `--since` flag now validates against alphanumeric, common git chars (/, -, :, @, ^, ~, !, +)

```bash
# This is now blocked:
svccat check --since "$(whoami)"
svccat check --since "HEAD; echo hacked"
```

### 🟠 High: SSRF Prevention
**Impact:** Probing of internal infrastructure, metadata service access
**Fix:** URL validation blocks private IPs and loopback addresses
**Details:** Blocks 127.x, 10.x, 172.16-31.x, 192.168.x for IPv4 and ::1, fe80::/10, fc00::/7 for IPv6

### 🟠 High: Deserialization Bomb Protection
**Impact:** Resource exhaustion attacks via YAML/TOML expansion
**Fix:** Resource limits on manifest complexity
- Max file size: 10 MB (prevents YAML anchor expansion)
- Max services: 10,000
- Max service name: 256 bytes
- Max dependencies: 1,000 per service

### 🟡 Medium: Path Traversal Prevention
**Impact:** Reading arbitrary files outside repo
**Fix:** Strict path validation on `path`, `submodule`, `docs`, `ci` fields
**Blocks:** Absolute paths, `..` sequences, null bytes

### 🟡 Medium: Symlink Attack Detection
**Impact:** Directory traversal, TOCTOU attacks
**Fix:** Rejects symlinks during service discovery
**Details:** Warns users and skips symlink entries

### 🟡 Medium: Glob Pattern DoS
**Impact:** Resource exhaustion via expensive patterns
**Fix:** Limits patterns to 20 total, max 2 consecutive wildcards
**Blocks:** `**/**/**`, unbounded pattern expansion

### 🟡 Medium: Information Disclosure Prevention
**Impact:** System path leakage in error messages
**Fix:** Path redaction module converts absolute paths to repo-relative
**Details:** Absolute paths shown as `[absolute path: /etc/passwd]` in logs

## What You Need To Do

### Users
1. **Update immediately:**
   ```bash
   cargo install svccat@0.19.0
   # or
   brew upgrade svccat
   ```

2. **Review your manifests:**
   - Check for any `../` sequences in path fields
   - Verify no absolute paths in `path`, `submodule`, `docs`, `ci`
   - Remove any symlinks from your service directories

3. **Check GitHub Actions:**
   - If using svccat in CI, the workflow updates are automatic
   - Verify `--since` uses valid git refs (branch names, tags, commit SHAs, HEAD, HEAD~N)

### Security Researchers
- Responsible disclosure process: See [SECURITY.md](https://github.com/rodmen07/svccat/blob/main/SECURITY.md)
- Report vulnerabilities to the maintainers privately
- Do NOT create public GitHub issues for security vulnerabilities

## Testing & Validation

This release has been thoroughly tested:
- ✅ 69 passing tests (17 unit + 52 integration)
- ✅ Clippy linting with zero warnings
- ✅ `cargo audit` with no vulnerabilities
- ✅ Cross-platform testing (Linux, macOS, Windows)
- ✅ Security validation tests for all 10 mitigations

## Documentation

**Complete security documentation:** [SECURITY.md](https://github.com/rodmen07/svccat/blob/main/SECURITY.md)

Topics covered:
- Threat model and attack vectors
- Detailed mitigation for each vulnerability
- Known limitations and pending work
- Responsible disclosure process
- Security best practices for users
- Safe defaults and secure by default design

## Questions?

- **Report a bug:** GitHub Issues
- **Report a security vulnerability:** See SECURITY.md
- **General questions:** GitHub Discussions

## Gratitude

This security release reflects months of thorough security review and testing. Thank you to everyone who uses svccat and trusts us with drift detection across your service catalog.

---

**svccat team**  
_Detecting drift. Preventing drift. Securing drift._
