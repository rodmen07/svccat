# Security Best Practices for svccat

This guide helps you use svccat securely and understand how it protects your service catalog.

## Table of Contents

1. [Input Validation](#input-validation)
2. [Git References](#git-references)
3. [Manifest Safety](#manifest-safety)
4. [URL Validation](#url-validation)
5. [Discovery & Symlinks](#discovery--symlinks)
6. [CI/CD Integration](#cicd-integration)
7. [Threat Mitigation](#threat-mitigation)

---

## Input Validation

### Never Trust User Input

svccat validates all inputs to prevent injection and path traversal attacks. However:

- ✅ **Safe:** Git refs from trusted branches (main, develop, releases/*)
- ❌ **Unsafe:** Git refs from user-submitted forms or environment variables
- ❌ **Unsafe:** Manifest paths from untrusted sources

### Using `--since` Safely

```bash
# ✅ Safe: Fixed git references
svccat check --since main
svccat check --since v0.19.0
svccat check --since HEAD~5
svccat check --since refs/heads/feature/drift

# ❌ Unsafe: Dynamic/untrusted refs
svccat check --since "$USER_INPUT"
svccat check --since "$(curl http://untrusted-api.com/ref)"
```

---

## Git References

### Valid Git References

svccat validates git references against a strict allowlist. Valid references include:

- Branch names: `main`, `develop`, `feature/xyz`
- Tags: `v1.0.0`, `release-2024-01-15`
- Commit SHAs: `abc1234de5f...`
- Special refs: `HEAD`, `HEAD~1`, `HEAD~5`
- Remote refs: `origin/main`, `upstream/develop`

### What Gets Blocked

These are rejected for security:

```bash
# Command injection attempts
--since "HEAD; echo hacked"
--since "$(whoami)"
--since "`cat /etc/passwd`"

# Path traversal attempts
--since "../../../etc/passwd"
--since "HEAD:../../sensitive"

# Null byte injection
--since "HEAD\0malicious"

# Excessively long refs (>256 chars)
--since "very_long_branch_name_" # (257+ chars)
```

---

## Manifest Safety

### Path Field Validation

All path fields in your manifest are validated:

```yaml
services:
  - name: api
    path: services/api              # ✅ Relative path OK
    # path: /etc/passwd             # ❌ Blocked: absolute path
    # path: ../../etc/passwd        # ❌ Blocked: path traversal
    
    docs: docs/README.md            # ✅ Relative path OK
    # docs: /absolute/path          # ❌ Blocked
    
    ci: .github/workflows/api.yml   # ✅ Relative path OK
    # ci: C:\Windows\System32       # ❌ Blocked: absolute path
    
    submodule: vendor/external-lib  # ✅ Relative path OK
```

### Manifest Size Limits

Manifests are protected against resource exhaustion:

- **Max file size:** 10 MB (prevents YAML expansion attacks)
- **Max services:** 10,000 per manifest
- **Max service name:** 256 bytes
- **Max dependencies:** 1,000 per service

If your manifest hits these limits:

```bash
# Error: manifest file is too large (11MB, max 10MB)
# Solution: Split into multiple manifest files

# Error: manifest has too many services (15000, max 10000)
# Solution: Use `--root` to run on subdirectories separately
```

### Safe Manifest Practices

```yaml
# ✅ Good: Fully qualified relative paths
discovery:
  paths:
    - services/*
    - microservices/*
    - "tools/*/app"  # Quotes optional, both work

services:
  - name: api
    path: services/api
    docs: services/api/README.md
    ci: services/api/.github/workflows/ci.yml

# ❌ Avoid: Absolute paths
services:
  - name: bad
    path: /services/api  # Rejected!

# ❌ Avoid: Path traversal
services:
  - name: bad
    path: services/../../../etc/passwd  # Rejected!

# ❌ Avoid: Symlinks
# (Discovery will skip symlinks with a warning)
```

---

## URL Validation

### Safe URLs for Webhooks & Pinging

svccat validates URLs to prevent Server-Side Request Forgery (SSRF) attacks.

```yaml
# Webhook configuration
[webhook]
url = "https://api.example.com/drift-webhook"  # ✅ Public HTTPS URL

# Service URLs for pinging
services:
  - name: api
    url = "https://api.example.com"  # ✅ Public HTTPS URL
    url = "http://api.example.com"   # ✅ HTTP OK for pinging
```

### What Gets Blocked (Webhook Only)

Webhooks enforce HTTPS for security (except localhost):

```yaml
[webhook]
url = "https://api.example.com/webhook"     # ✅ HTTPS OK

# ❌ HTTP blocked for webhooks
url = "http://api.example.com/webhook"      # Rejected!

# ✅ Exception: localhost for development
url = "http://localhost:3000/webhook"       # OK (dev only!)
```

### Blocked IP Addresses (All URLs)

These are rejected for all URL operations (both webhooks and pinging):

```yaml
services:
  # IPv4 private ranges - ALL BLOCKED
  - name: internal-api
    url = "http://192.168.1.1"              # ❌ Private
    url = "http://10.0.0.1"                 # ❌ Private
    url = "http://172.16.0.1"               # ❌ Private
    url = "http://127.0.0.1"                # ❌ Loopback
    url = "http://169.254.1.1"              # ❌ Link-local

  # IPv6 private ranges - ALL BLOCKED
  - name: ipv6-service
    url = "http://[::1]"                    # ❌ IPv6 loopback
    url = "http://[fe80::1]"                # ❌ IPv6 link-local
    url = "http://[fc00::1]"                # ❌ IPv6 ULA private
```

### Safe URL Practices

```bash
# ✅ Use public URLs only
svccat ping --url https://api.prod.company.com

# ✅ Development: use localhost exception
# Set webhook URL to http://localhost:3000 for local testing

# ✅ Use environment variables for secrets in URLs
# WEBHOOK_URL=https://api.example.com/webhook?token=$SECRET
```

---

## Discovery & Symlinks

### Service Discovery Safety

During service discovery, svccat:
- ✅ Scans directories matching glob patterns
- ✅ Looks for marker files (Cargo.toml, package.json, Dockerfile, etc.)
- ✅ Rejects symlinks with a warning
- ✅ Limits patterns to 20 total, 2 consecutive wildcards

### Handling Symlinks

Symlinks are skipped during discovery to prevent:
- Directory traversal attacks
- Time-of-check-time-of-use (TOCTOU) race conditions
- Infinite loops from circular symlinks

```bash
# If you see this warning:
# warning: skipping symlink './services/api' during discovery
#          (potential security risk)

# Solution: Replace symlinks with actual directories or
# use the 'path' field in your manifest to reference symlinks:

services:
  - name: api
    path: services/api  # Path can reference the actual location
```

### Safe Discovery Patterns

```yaml
discovery:
  # ✅ Safe: Specific glob patterns
  paths:
    - services/*
    - microservices/*
    - apps/*

  # ✅ Safe: Nested patterns with reasonable depth
  paths:
    - "services/*/app"
    - "teams/*/services/*"

  # ❌ Avoid: Unbounded wildcards (too many matches)
  paths:
    - "**"              # Matches everything!
    - "**/*"            # Extremely expensive

  # ❌ Avoid: Patterns with excessive wildcards
  paths:
    - "*/*/*/*/*/*"     # Too many consecutive wildcards
    - "**/**/**"        # Too many globstar patterns
```

---

## CI/CD Integration

### GitHub Actions Security

svccat in GitHub Actions automatically validates everything:

```yaml
# ✅ Safe: Using fixed branch names
- name: Check drift since main
  run: svccat check --since main

# ✅ Safe: Using GitHub context variables
- name: Check drift since base
  run: svccat check --since origin/${{ github.base_ref }}

# ❌ Avoid: Using arbitrary input without validation
- name: Check drift since user input
  run: svccat check --since "${{ github.event.comment.body }}"
```

### Secure Webhook Usage in CI

```yaml
# ✅ Safe: Store URLs in secrets
env:
  WEBHOOK_URL: ${{ secrets.DRIFT_WEBHOOK_URL }}

# ✅ Safe: Webhook runs only after successful validation
- name: Fire webhook
  if: success()
  run: svccat webhook --url "$WEBHOOK_URL"

# ❌ Avoid: Storing URLs in code
run: svccat webhook --url "https://secret-url-here.com"
```

### Audit Trail

Enable detailed logging for security audits:

```bash
# Enable debug output for investigation
RUST_LOG=debug svccat check

# Capture output for security review
svccat check --format markdown > drift-report.md
```

---

## Threat Mitigation

This section maps each threat to how svccat protects you:

| Threat | Attack Vector | svccat Protection |
|--------|---------------|-------------------|
| **Git Injection** | Malicious refs in `--since` | Strict whitelist validation |
| **SSRF** | Webhook/ping to internal IPs | IP address blocklist (private ranges) |
| **Deser. Bomb** | Large YAML with anchors | 10 MB file size limit |
| **Path Traversal** | `../` in manifest paths | Path validation rejects `..` |
| **Symlink Attack** | Symlinks to sensitive files | Symlink rejection during discovery |
| **Glob DoS** | Expensive patterns | Pattern limits (20 total, 2 consecutive wildcards) |
| **Info Disclosure** | Absolute paths in errors | Path redaction in error messages |
| **IPv6 Bypass** | IPv6 loopback in URLs | IPv6 private range detection |

### Reporting Security Issues

If you discover a security vulnerability:

1. **Do NOT** create a public GitHub issue
2. **Do** follow the [Responsible Disclosure process](SECURITY.md#responsible-disclosure)
3. **Report privately** to the maintainers

---

## Summary

### Remember:
- ✅ Use fixed, trusted git references
- ✅ Use relative paths in manifests (no `..` or `/`)
- ✅ Use public URLs for webhooks (HTTPS required)
- ✅ Let svccat validate everything else
- ✅ Report security issues privately

### Resources:
- [SECURITY.md](SECURITY.md) - Threat model and detailed mitigations
- [CHANGELOG.md](CHANGELOG.md) - v0.19.0 security fixes
- GitHub Issues - Bug reports and feature requests
- GitHub Discussions - Questions and community help

---

**Last Updated:** May 28, 2026 (v0.19.0)  
**svccat Security Team**
