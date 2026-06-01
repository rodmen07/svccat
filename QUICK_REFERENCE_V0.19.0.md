# Quick Reference: svccat v0.19.0

Fast lookup guide for common tasks and security considerations.

## Installation

```bash
cargo install svccat --version 0.19.0
svccat --version
```

---

## Common Commands

### Check for Drift

```bash
# In your repository
svccat check

# Specify manifest file
svccat check -m path/to/manifest.yaml

# Check specific git reference
svccat check -s main

# Verbose output
svccat check -v
```

### Check Current Status

```bash
# Show what svccat found
svccat status

# JSON output
svccat status --json
```

### Watch Mode

```bash
# Watch for changes
svccat watch

# With custom interval (milliseconds)
svccat watch --interval 5000
```

---

## Security: Do's & Don'ts

### Git References

✅ **VALID patterns:**
```
main
develop
v1.0.0
feature/auth-redesign
HEAD~1
refs/heads/main
```

❌ **INVALID patterns:**
```
main; echo hacked          # Command injection
$(whoami)                  # Command substitution
`git log`                  # Backtick substitution
main && git push           # Multiple commands
HEAD~$(echo 1)             # Nested substitution
```

### Webhooks & URLs

✅ **VALID:**
```yaml
webhooks:
  - url: "https://api.example.com/drift"
  - url: "https://alerts.internal.company.com/svccat"
```

❌ **INVALID:**
```yaml
webhooks:
  - url: "http://127.0.0.1:8080/hook"      # Localhost rejected
  - url: "http://10.0.0.5/hook"            # Private IP rejected
  - url: "http://192.168.1.1/hook"         # Private IP rejected
  - url: "http://example.com/hook"         # HTTP not HTTPS (rejected)
```

**Exception:** Localhost works in development (use `http://localhost:8080`)

### Manifest Paths

✅ **VALID:**
```yaml
services:
  - name: api
    path: "src/api"
  - name: web
    path: "web/frontend"
```

❌ **INVALID:**
```yaml
services:
  - name: api
    path: "/etc/passwd"              # Absolute path rejected
  - name: api
    path: "../../etc/passwd"         # Path traversal rejected
  - name: api
    path: "C:\\Windows\\System32"   # Windows absolute rejected
```

### Discovery Patterns

✅ **GOOD:**
```yaml
discovery:
  paths:
    - "services/*"
    - "apps/*/service"
    - "**/*.service.yaml"
```

❌ **PROBLEMATIC:**
```yaml
discovery:
  paths:
    - "services/*"
    - "services01/*"
    - "services02/*"
    - # ... (30 total) - TOO MANY (limit: 20)

  # Or excessive wildcards:
  - "**/**/**/src"  # TOO MANY (limit: 2 consecutive)
```

---

## Configuration Template

### Minimal (Single Repo)

```yaml
version: "1"

discovery:
  paths:
    - "services/*"

services:
  - name: api
    language: Rust
    platform: "Cloud Run"
```

### Recommended (Production)

```yaml
version: "1"

discovery:
  paths:
    - "services/*"
    - "lib/*/service"

services:
  - name: api
    language: Rust
    platform: "Cloud Run"
    repository: "services/api"
    team: "Platform"

webhooks:
  - url: "https://alerts.company.com/svccat"
    events:
      - "drift_detected"
      - "discovery_complete"

slack:
  webhook_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
  channel: "#deployments"
  mention_on_drift: true
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Service Drift Detection

on:
  push:
    branches: [main, develop]
  pull_request:
  schedule:
    - cron: '0 10 * * *'  # Daily at 10 AM UTC

jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4.1.7

      - name: Install svccat
        run: cargo install svccat --version 0.19.0

      - name: Check for drift
        run: svccat check

      - name: Report status
        if: always()
        run: svccat status --json > drift-report.json

      - name: Upload report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: drift-report
          path: drift-report.json
```

### GitLab CI

```yaml
drift_check:
  stage: validate
  image: rust:latest
  script:
    - cargo install svccat --version 0.19.0
    - svccat check
    - svccat status --json > drift-report.json
  artifacts:
    paths:
      - drift-report.json
    expire_in: 7 days
```

---

## Error Messages & Solutions

### "Git reference invalid"

```
Error: Invalid git reference: main; echo hacked
```

**Cause:** Git reference contains shell metacharacters  
**Solution:** Use plain branch names, tags, or safe git syntax

```bash
# ✅ Correct
svccat check -s main
svccat check -s v1.0.0
svccat check -s HEAD~1

# ❌ Wrong
svccat check -s 'main; echo hacked'
```

---

### "URL validation failed: private IP address"

```
Error: Webhook URL rejected: Private IP address 127.0.0.1
```

**Cause:** Webhook points to private/internal IP  
**Solution:** Use public URLs or localhost in development

```yaml
# ✅ Production
webhooks:
  - url: "https://api.example.com/webhooks/svccat"

# ✅ Development
webhooks:
  - url: "http://localhost:8080/webhooks/svccat"

# ❌ Never allowed
webhooks:
  - url: "http://10.0.0.5/webhooks"
  - url: "http://192.168.1.1/webhooks"
```

---

### "Path traversal detected"

```
Error: Path traversal detected: ../../etc/passwd
```

**Cause:** Manifest contains ".." or absolute paths  
**Solution:** Use relative paths within repo

```yaml
# ✅ Correct
services:
  - name: api
    path: "src/api"

# ❌ Wrong
services:
  - name: api
    path: "../../etc/passwd"    # Path traversal
  - name: api
    path: "/etc/passwd"         # Absolute path
```

---

### "Manifest too large"

```
Error: Manifest exceeds size limit: 11.5 MB > 10 MB
```

**Cause:** Manifest file exceeds 10 MB limit  
**Solution:** Split manifest or reduce service definitions

```bash
# Check manifest size
ls -lh services.yaml

# If too large, split into multiple files:
# - services/api.yaml (split by domain)
# - services/web.yaml
# - services/infra.yaml
```

---

### "Excessive glob patterns"

```
Warning: Discovery has 25 paths (limit: 20)
```

**Cause:** Too many discovery patterns (limit: 20)  
**Solution:** Consolidate patterns

```yaml
# ❌ Too many patterns
discovery:
  paths:
    - "services01/*"
    - "services02/*"
    - "services03/*"
    # ... (25 total)

# ✅ Consolidated
discovery:
  paths:
    - "services*/*"  # Single pattern covers all
```

---

### "Symlink skipped during discovery"

```
Warning: Skipped symlink: services/legacy -> ../old-repo
```

**Cause:** Discovery found and skipped a symlink  
**Solution:** Either remove symlink or list service explicitly

```yaml
# Option 1: Remove symlink from repo
rm services/legacy

# Option 2: Explicitly list the service
services:
  - name: legacy
    path: "legacy-service"
    language: Python
    platform: "Heroku"
```

---

## Testing Your Setup

### Quick Validation

```bash
# Test basic functionality
svccat check
svccat status
svccat ci

# Test specific features
svccat check -s main         # Git ref validation
svccat check -v              # Verbose output
```

### Security Validation

```bash
# Test git injection prevention
svccat check -s 'main; echo hacked'  # Should FAIL
svccat check -s 'main'               # Should PASS

# Test path validation
# (Create test manifest with absolute path)
echo 'services: [{name: test, path: /etc/passwd}]' > test.yaml
svccat check -m test.yaml            # Should FAIL
rm test.yaml

# Test URL validation
# (Create test with private IP webhook)
echo 'webhooks: [{url: "http://127.0.0.1:8080"}]' > test.yaml
svccat check -m test.yaml            # Should FAIL
rm test.yaml
```

---

## Performance Tips

1. **Limit discovery patterns:** Keep under 10 for best performance
2. **Reasonable manifest size:** Stay well under 10 MB limit
3. **Avoid excessive wildcards:** Use `*` instead of `**` when possible
4. **Cache manifests:** In CI, cache parsed manifests when possible

---

## Debugging

### Verbose Output

```bash
svccat check -v           # Show detailed progress
svccat check -vv          # Extra verbose (debug level)
```

### JSON Output

```bash
svccat status --json      # Machine-readable output
svccat check --json       # Drift in JSON format
```

### Check Configuration

```bash
# View discovered services
svccat list

# Show manifest details
svccat show services.yaml
```

---

## Documentation Links

| Document | Purpose |
|----------|---------|
| [SECURITY.md](SECURITY.md) | Threat model & vulnerabilities |
| [SECURITY_BEST_PRACTICES.md](SECURITY_BEST_PRACTICES.md) | Safe usage patterns |
| [README.md](README.md) | Getting started |
| [CHANGELOG.md](CHANGELOG.md) | Release history |
| [FUZZING.md](FUZZING.md) | Fuzz testing guide |

---

## Getting Help

### Report a Bug

```bash
# Create security issue
# .github/ISSUE_TEMPLATE/security_report.md

# Create bug report
# https://github.com/rodmendoza07/svccat/issues
```

### Request a Feature

```bash
# Use feature request template
# .github/ISSUE_TEMPLATE/feature_request.md
```

### Community Discussion

```bash
# Join feedback discussion
# https://github.com/rodmendoza07/svccat/discussions
```

---

## Version Info

```bash
svccat --version
# svccat 0.19.0

svccat --help
# Show all commands
```

---

**Last Updated:** May 28, 2026  
**Version:** 0.19.0
