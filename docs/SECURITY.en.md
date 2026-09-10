# Security Policy

> [中文版本](../SECURITY.md) | English

## Supported versions

| Version / branch | Security maintenance |
|---|---|
| Latest code on `main` | Reports accepted; confirmed problems addressed |
| Historical releases / old branches | No separate maintenance or guaranteed backports |

Reports against older versions are still welcome. State the version and whether `main` is affected if known; upgrading or reproducing against real data is not required to report. Fixes generally land on `main` before a maintainer schedules a release. No fixed remediation timeline is promised.

## Report privately

**Do not disclose an unpatched vulnerability, exploitable PoC, or sensitive evidence in a public issue, PR, discussion, or attachment.**

1. If the repository's Security page offers **Report a vulnerability**, use [GitHub private vulnerability reporting](https://github.com/lucan6290/skills-hub/security/advisories/new)
2. If that option is unavailable, disabled, or inaccessible, email `lucan6290@gmail.com` with a subject such as `[Security] Skills Hub - short description`

This document does not assert that private reporting is enabled; repository administrators manage that GitHub setting. For ordinary questions, see [Support](SUPPORT.en.md).

## What to include

- Affected version/commit, OS, and architecture
- Description, potential impact, required privileges, and user interaction
- Minimal reproduction or PoC with synthetic data, including expected and actual results
- Import source, target tool, global/project scope, path, or file type where relevant
- Redacted log excerpts and possible remediation ideas, if available
- Contact details and your preference for public credit or anonymity

Do not include real tokens, private keys, databases, third-party personal information, or data obtained without permission. If evidence cannot be shared safely, describe the issue first and agree on the necessary evidence with the maintainer.

## Scope and boundaries

Relevant issues include path traversal, unintended filesystem access, symlink handling, command injection, Tauri permission bypasses, credential exposure, and update authenticity verification.

- User-imported skills and prompts do not become trustworthy merely because Skills Hub manages them; review their sources and contents
- Malicious skills, repository files, links, or external inputs that bypass intended path/permission boundaries are in scope; local execution alone does not exclude a vulnerability
- Report dependency vulnerabilities that affect this project with the affected version and actual usage context; also consider notifying upstream
- UI defects, ordinary crashes, performance issues, and feature proposals generally belong in public issues. When uncertain about security impact, report privately first

Use non-destructive tests only in environments you own or are authorized to test. Do not access others' data or disrupt their systems. This policy does not authorize testing beyond the project's control or promise a bounty.

## Handling and coordinated disclosure

1. Maintainers will acknowledge and assess reports as capacity allows and may request more information; there is no fixed response or remediation SLA
2. Coordinate disclosure until a fix or workable mitigation is ready; avoid publishing exploitable details prematurely
3. Confirmed issues will be prioritized by impact, with affected versions and upgrade/mitigation guidance in an advisory or changelog when disclosed
4. Obtain the reporter's consent before public credit; private contact details are not published by default. Please state if you prefer anonymity

## Development and release security

- Never commit `.env` files, tokens, signing private keys/passwords, or unredacted logs/data
- Ordinary development does not need the release signing key; maintainers manage it through the release process
- Do not bypass authentication, path validation, or update-signature checks as a workaround
- Revoke or rotate exposed credentials and notify maintainers privately; deleting files or rewriting history does not revoke credentials
- Verify installers, signatures, and update metadata before publishing; see the [release workflow](release-workflow.md) (Chinese full guide)
