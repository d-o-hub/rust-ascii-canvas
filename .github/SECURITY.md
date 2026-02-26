# Security Policy

## Supported Versions

The following versions of ASCII Canvas are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please send an email to the maintainers. We appreciate responsible disclosure and will work to address the issue promptly.

### What to Include

1. Description of the vulnerability
2. Steps to reproduce the issue
3. Potential impact assessment
4. Any suggested fixes (optional)

### Response Timeline

We aim to acknowledge vulnerability reports within 48 hours and provide a timeline for remediation based on severity:

- **Critical**: 7 days
- **High**: 14 days
- **Medium**: 30 days
- **Low**: 60 days

## Security Best Practices

- Keep dependencies up to date
- Review code that processes user input
- Follow Rust's safety guidelines

## Dependencies

This project uses the following main dependencies:
- wasm-bindgen
- web-sys
- serde

We regularly audit dependencies for known vulnerabilities.
