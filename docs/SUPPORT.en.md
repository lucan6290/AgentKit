# Support Guide

> [中文版本](../SUPPORT.md) | English

This guide covers installation, usage, and troubleshooting. See [Contributing](CONTRIBUTING.en.md) for development and the [Security Policy](SECURITY.en.md) for private vulnerability reports.

## Where to ask

| Topic | Channel |
|---|---|
| Installation, usage, or sync | Read the [README](README.en.md) and search [Issues](https://github.com/lucan6290/agentkit/issues), then open a usage question if needed |
| Reproducible bug | Use the [bug report form](https://github.com/lucan6290/agentkit/issues/new?template=bug_report.yml) |
| Feature proposal | Use the [feature request form](https://github.com/lucan6290/agentkit/issues/new?template=feature_request.yml); explain the use case and alternatives |
| Vulnerability or exposed credentials | Follow [SECURITY.en.md](SECURITY.en.md), not a public issue |
| Harassment or community conduct | Use the private channel in the [Code of Conduct](CODE_OF_CONDUCT.en.md) |

Use public issues for ordinary questions so others can learn from the outcome. This is a community-maintained project; fixed response times, commercial support, and implementation of every feature request are not guaranteed. Chinese and English are welcome.

## Before reporting

1. Record the application version or build commit, OS, and architecture; review [Releases](https://github.com/lucan6290/agentkit/releases) and the [changelog](../CHANGELOG.md)
2. Search for duplicates and add useful environment details or a minimal reproduction to an existing issue
3. Reproduce with a disposable skill and temporary directory; identify whether installation, import, sync, editing, or the target tool is failing
4. For sync issues, record the target tool, global/project scope, path structure, and symlink/junction/copy mode if known
5. For build errors, include Node.js / npm / Rust versions and the failing command; check [development prerequisites](CONTRIBUTING.en.md)

Do not treat deleting all application data, clearing databases, disabling permission or update-signature checks, or running everything as administrator as routine troubleshooting. Back up data and understand the scope before restoring, importing backups, removing, or overwriting files.

## Include in your report

- A short summary and minimal reproducible steps
- Expected and actual behavior
- App version, OS, and architecture; include commit and toolchain versions for development builds
- Target tool, sync scope, and representative paths with personal/project names removed
- Relevant error text, a small redacted log excerpt, or screenshots—not a full database or workspace
- Workarounds tried, whether the issue is intermittent, and results in an isolated directory

## Privacy and logs

Inspect attachments for tokens, API keys, passwords, private keys, email addresses, personal paths, private repository URLs, and confidential prompt/skill content. Never upload `.env` or signing files.

If sensitive evidence is needed, contact the maintainer privately to agree on minimal information and a transfer method. If credentials were exposed, stop sharing them and revoke or rotate them; deleting public content alone is insufficient.

## Downloads and updates

- Prefer attachments officially published by maintainers in this repository's [Releases](https://github.com/lucan6290/agentkit/releases), not unknown repackaged binaries
- The current release workflow builds Windows NSIS and MSI installers. macOS/Linux development requires platform prerequisites and a local build; this does not promise prebuilt releases for those platforms
- Keep the error details when automatic updates fail. After checking the source and version, a published installer can be used for manual installation; do not bypass signature verification
- Missing assets or a Draft Release do not mean a release is ready. Maintainers should follow the [release workflow](release-workflow.en.md)
