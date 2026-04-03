# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.x.x (current development) | Yes |

As the project matures, this table will be updated to reflect the support window for stable releases.

## Reporting Vulnerabilities

**Do NOT file public GitHub issues for security vulnerabilities.**

Instead, report vulnerabilities privately:

- **Email:** andbubnov@gmail.com
- **Subject line:** `[SECURITY] <brief description>`
- **Encrypt with PGP** if possible (key available upon request)

### What to Include

- Description of the vulnerability
- Steps to reproduce (minimal `.sno` code if applicable)
- Potential impact assessment
- Suggested fix (if you have one)
- Your name/handle for credit (optional)

### Response Timeline

| Action | Timeframe |
|--------|-----------|
| Acknowledgment of report | 72 hours |
| Initial assessment | 7 days |
| Fix development | Varies by severity |
| Coordinated disclosure | 90 days from report |

We follow a 90-day coordinated disclosure policy. If a fix is available sooner, we will coordinate an earlier disclosure with the reporter.

## Scope

The following are **in scope** for security reports:

- **Compiler bugs** that generate unsafe or incorrect native code
- **Memory safety issues** in the compiler itself (buffer overflows, use-after-free, etc.)
- **Tooling vulnerabilities** — constrained decoding bypass, GBNF injection, module interface manipulation
- **Supply chain issues** — dependency vulnerabilities affecting the compiler
- **Code execution** — unexpected code execution during compilation (not runtime)

## Out of Scope

The following are **not** security vulnerabilities:

- Bugs in user-written Synoema code (that's a language design issue)
- Performance issues or denial-of-service via slow compilation (report as regular bugs)
- Vulnerabilities in dependencies that don't affect Synoema's usage of them
- Social engineering attacks against project maintainers

## Severity Classification

| Severity | Description | Example |
|----------|-------------|---------|
| Critical | Remote code execution, memory corruption in generated code | Compiler generates code that corrupts memory |
| High | Information disclosure, privilege escalation | Compiler reads files outside project directory |
| Medium | Denial of service, limited impact bugs | Compiler crash on crafted input |
| Low | Minor issues, theoretical attacks | Timing side-channels in compilation |

## Recognition

We gratefully acknowledge security researchers who responsibly report vulnerabilities. With your permission, we will credit you in the release notes and in a SECURITY_ACKNOWLEDGMENTS.md file.

## Changes

This security policy may be updated as the project evolves. Check this file for the latest version.
