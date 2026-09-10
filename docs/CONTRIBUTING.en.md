# Contributing

> [中文版本](../CONTRIBUTING.md) | English

Thank you for contributing to Skills Hub! Bug fixes, tests, documentation, translations, and well-scoped feature proposals are welcome. Please follow our [Code of Conduct](CODE_OF_CONDUCT.en.md).

## Choose the right channel

- Usage and troubleshooting: read the [Support Guide](SUPPORT.en.md) and [README](README.en.md)
- Reproducible bugs: search [existing Issues](https://github.com/lucan6290/skills-hub/issues) before submitting a bug form
- Features or substantial design changes: describe the problem, use case, and alternatives in an issue before implementing
- Security vulnerabilities: follow the [Security Policy](SECURITY.en.md) privately; do not open a public issue or publish a PoC
- Small documentation fixes can go straight to a PR; Chinese and English are welcome in issues and discussions

## Local development

### Prerequisites

- Git, Node.js, and npm. Node.js 22.12+ in the 22.x series aligns with the release workflow; see [Vite requirements](https://vite.dev/guide/)
- Rust stable. `Cargo.toml` declares 1.77.2 as the minimum, but locked dependencies may need a newer toolchain
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS, including C++ build tools and WebView2 on Windows

Fork the repository, clone your fork, and run from the repository root:

```bash
git switch -c codex/describe-your-change
cd frontend
npm ci
npm run tauri dev
```

The branch name is an example. Use the existing npm lockfile; do not delete it or broadly upgrade dependencies merely to bypass an installation error.

`npm run tauri dev` starts both Vite and the desktop window. `npm run dev` starts only the frontend and cannot validate native Tauri behavior in a browser. Everyday development does not require a release signing key; do not request or replace upstream keys.

**Protect your data:** imports, sync, removal, file editing, and database maintenance may change local files. Use disposable skills, isolated directories, and backups rather than your everyday tool directories or only copies.

## Engineering conventions

Read [AGENTS.md](../AGENTS.md), then the [frontend](../frontend/AGENTS.md) or [Rust](../frontend/src-tauri/AGENTS.md) rules and any rules below the target directory.

- Keep changes focused; avoid unrelated refactoring, repository-wide formatting, or dependency updates
- Use `snake_case` for cross-boundary DTOs, JSON, and Tauri command parameters; use `camelCase` for frontend internal state/functions. See [naming conventions](naming-conventions.md)
- Follow the existing feature layout, path aliases, and Rust commands / services / repositories separation
- Update both UI languages and matching `docs/*.en.md` documents; historical changelog entries remain Chinese
- You are responsible for reviewing, understanding, and validating AI-assisted contributions
- Preserve attribution and verify the origin and license of third-party code and assets. The project retains its [MIT License](../LICENSE); importing third-party content does not relicense it as MIT

## Validation

Run commands from the repository root:

| Change | Verification |
|---|---|
| Frontend logic, components, styles, or types | `npm --prefix frontend run check` (ESLint, TypeScript, and Vite) |
| Rust or shared contracts | `cargo test --manifest-path frontend/src-tauri/Cargo.toml`; shared changes also need frontend checks |
| Version/release documentation | `node scripts/version.mjs check` and extraction for a real version, e.g. `node scripts/extract-changelog.mjs v0.2.3` |
| Documentation/templates only | Relative links, language navigation, commands, form fields, and `git diff --check` |
| UI or filesystem sync | Screenshots or reproduction results with OS, tool version, and global/project scope |

Reproduce bugs first and add regression coverage where feasible. State which platforms and symlink / junction / copy or non-ASCII path scenarios you tested. Report checks you could not run and why; never present them as passed.

The current [CI](../.github/workflows/ci.yml) runs frontend lint and build for PRs and pushes to `main`. It **does not run Rust tests**, and a green CI result does not replace native or desktop verification.

## Commits and changelog

Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) prefixes with the project's Chinese summary convention: `类型: 简要描述`. Examples: `docs: 补充贡献指南` and `fix: 修复中文路径下的技能导入`.

| Prefix | Purpose |
|---|---|
| `feat:` / `fix:` | Features / bug fixes |
| `docs:` | Documentation or translation |
| `refactor:` / `perf:` / `style:` | Refactoring, performance, styles/formatting |
| `test:` | Tests |
| `build:` / `ci:` | Build and CI |
| `chore:` | Dependencies or maintenance |

Keep each commit to one related unit of work. Explain breaking changes and migration steps in a `BREAKING CHANGE:` footer. Maintainers use `release: vX.Y.Z` for release commits, which are not changelog entries themselves.

Record notable changes under `Unreleased` in [CHANGELOG.md](../CHANGELOG.md), using Added / Changed / Deprecated / Removed / Fixed / Security. Technical is a project-specific extension for documentation, build, and maintenance work. Do not rewrite historical entries or bump versions/create tags in an ordinary PR.

## Pull requests

1. Make a focused change on a feature branch, run relevant checks, and inspect `git diff` and `git diff --check`
2. Stage only your contribution and inspect `git diff --cached` for unrelated work, generated output, personal data, or secrets
3. Push to your own fork and open a PR against this repository's `main`. Agents must obtain explicit authorization before pushing
4. Complete the [PR template](../.github/pull_request_template.md): rationale, issue, scope, actual verification commands/results, screenshots, and compatibility or migration risks
5. Respond to maintainer review; use a Draft PR for early feedback. Maintainers confirm merges and releases

Never commit `.env` files, tokens, private keys, signing passwords, databases, or unredacted logs. Follow the security policy for an accidental disclosure; deleting a file does not revoke credentials.

## Maintainer references

- [Release workflow](release-workflow.md) (中文完整版): versions, gates, signing, draft releases, and recovery
- [Security policy](SECURITY.en.md): support scope, private reporting, and coordinated disclosure
- [Support guide](SUPPORT.en.md): routing, diagnostics, and data protection
