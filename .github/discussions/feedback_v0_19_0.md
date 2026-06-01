# svccat v0.19.0 Security Update - Feedback Thread

Thank you for using svccat! We're excited to share the security improvements in v0.19.0.

## What's New

We've addressed **10 critical and high-severity security vulnerabilities**:

- ✅ **Git command injection prevention** - Strict ref validation with allowlist patterns
- ✅ **Server-Side Request Forgery (SSRF) prevention** - Private IP blocking and URL validation
- ✅ **Deserialization bomb protection** - Resource limits on manifest size and complexity
- ✅ **Path traversal prevention** - Directory escape detection and validation
- ✅ **Symlink attack prevention** - Metadata-based symlink detection
- ✅ **Glob pattern DoS prevention** - Pattern count and wildcard limits
- ✅ **Information disclosure prevention** - Path redaction in error messages
- ✅ **IPv6 support** - Proper handling of IPv6 private ranges
- ✅ **Cross-platform compatibility** - Windows backslash handling in git commands
- ✅ **Code quality improvements** - 11 clippy warnings resolved

**See full details:** [SECURITY.md](https://github.com/rodmendoza07/svccat/blob/main/SECURITY.md)  
**Best practices guide:** [SECURITY_BEST_PRACTICES.md](https://github.com/rodmendoza07/svccat/blob/main/SECURITY_BEST_PRACTICES.md)

## We Want Your Feedback

Please share your experience:

### ✅ What worked well?
- Which features are you using?
- Did the security improvements affect your workflow?
- Are the new validation rules reasonable for your use cases?

### 🐛 Found an issue?
- Describe the problem
- What command triggered it?
- What was your manifest/configuration?
- (See [Responsible Disclosure](https://github.com/rodmendoza07/svccat/blob/main/SECURITY.md#responsible-disclosure) for security issues)

### 💡 Suggestions for v0.20.0
- What features would help you most?
- Should we add multi-repo support?
- Custom validation rules?
- Integration with other tools?

---

**Version:** 0.19.0  
**Released:** May 28, 2026  
**Requires upgrade:** Yes (addresses critical security issues)
