---
name: Security Issue Report
about: Report a security vulnerability in svccat
title: "[SECURITY] Brief description"
labels: security
---

## Vulnerability Description

**Type:** (e.g., SSRF, Path Traversal, Git Injection, etc.)

**Severity:** (Critical / High / Medium / Low)

**Affected Version:** (e.g., 0.19.0 or earlier)

### Description

[Describe the security issue clearly and concisely]

### Proof of Concept

[Provide minimal steps to reproduce, including manifest/config examples]

```yaml
# Example manifest that triggers the issue
discovery:
  paths: ["services/*"]
services:
  - name: test
    # Your config here
```

### Expected Behavior

[What should happen instead]

### Environment

- **svccat version:** 
- **OS:** (Linux / macOS / Windows)
- **Git version:** 
- **Other relevant details:** 

---

## Responsible Disclosure

⚠️ **Please do not publicly disclose security vulnerabilities.** For critical issues, email security@[domain] or check [SECURITY.md](../SECURITY.md) for responsible disclosure guidelines.
