# Security Report

## Known Vulnerabilities

### RUSTSEC-2024-0437 - Protobuf Vulnerability

**Status**: ACCEPTED (Temporary)  
**Severity**: Critical  
**Affected Dependency**: `protobuf 2.28.0`  
**Vulnerability Path**: `protobuf 2.28.0 ← prometheus 0.13.4 ← pingora-core 0.6.0 ← pingora 0.6.0`

**Description**: 
The protobuf crate version 2.28.0 has a vulnerability that can cause crashes due to uncontrolled recursion. This vulnerability is present in our dependency chain through the Pingora framework.

**Impact Assessment**:
- The vulnerability affects the protobuf parsing functionality
- Our application uses Pingora for HTTP proxying, which internally uses prometheus for metrics
- The risk is mitigated by the fact that we control the input to our web server

**Mitigation Strategies**:
1. **Current**: Monitor for Pingora updates that include newer prometheus/protobuf versions
2. **Runtime**: Input validation and rate limiting reduce exploitation risk
3. **Monitoring**: Automated security scanning via GitHub Actions weekly

**Resolution Plan**:
- [ ] Monitor Pingora releases for versions > 0.6.0
- [ ] Check for prometheus crate updates in pingora-core
- [ ] Consider alternative HTTP proxy frameworks if vulnerability persists
- [ ] Weekly security audit via GitHub Actions

**Date Identified**: December 2024  
**Next Review**: Weekly via automated security workflow

---

## Unmaintained Dependencies (Warnings)

The following dependencies are flagged as unmaintained but pose lower risk:

1. **atty 0.2.14** - Terminal detection (indirect via clap)
2. **derivative 2.2.0** - Derive macro utilities  
3. **paste 1.0.15** - Macro utilities
4. **proc-macro-error 1.0.4** - Procedural macro error handling
5. **yaml-rust 0.4.5** - YAML parsing (indirect via serde_yaml)

These are all indirect dependencies through the Pingora framework and don't pose immediate security risks to our application.

---

## Security Contacts

For security issues, please create an issue in the repository with the "security" label.
