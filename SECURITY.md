# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.2.x   | ✅ |
| < 0.2   | ❌ |

## Reporting a Vulnerability

**Do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via:

- **GitHub Private Vulnerability Reporting**: [Submit a report](https://github.com/epicsagas/claudy/security/advisories/new)

### Response Times

| Stage | Target |
|-------|--------|
| Acknowledgement | Within 48 hours |
| Initial assessment | Within 5 business days |
| Patch / fix | Within 90 days |

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Suggested fix (if available)

### Disclosure Policy

- Coordinated disclosure: we will work with you to fix the issue before any public announcement.
- Credit will be given to reporters unless they request anonymity.

## Security Best Practices for Users

- Store API keys in `~/.claudy/secrets.env` (file permissions `0600`).
- Never commit `secrets.env` to version control.
- Use `claudy doctor` to verify your configuration health.
- Keep claudy updated: `claudy update`.
