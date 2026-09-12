# Release Workflow

> [中文完整版](release-workflow.md) | English

This document summarizes the release gates for AgentKit. The Chinese guide is the source of truth and contains the complete PowerShell commands and recovery procedures.

## Release checklist

1. **Preflight** — Work from the repository root, confirm a clean `main` branch, verify the `origin` remote, and make sure no private keys, credentials, generated artifacts, or unrelated changes are staged.
2. **Choose the version** — Follow Semantic Versioning and inspect the changes since the previous tag. Confirm the MAJOR/MINOR/PATCH decision with the maintainer before changing files.
3. **Quality gate** — Run `npm ci` and `npm --prefix frontend run check`. Run `cargo test --manifest-path frontend/src-tauri/Cargo.toml` and a release Rust build when the release guide requires it.
4. **Update the changelog** — Add a dated entry below `## [Unreleased]` in `CHANGELOG.md` using Added, Changed, Deprecated, Removed, Fixed, Security, or the project's Technical category. Keep release notes user-facing and do not rewrite historical entries.
5. **Synchronize versions** — Run `node scripts/version.mjs set X.Y.Z`, then `node scripts/version.mjs check`. This keeps `frontend/package.json`, `frontend/package-lock.json`, and `frontend/src-tauri/Cargo.toml` aligned.
6. **Verify release notes** — Run `node scripts/extract-changelog.mjs vX.Y.Z` and confirm the output is the intended section.
7. **Commit and tag** — Inspect `git diff` and `git diff --check`, create the single release commit `release: vX.Y.Z`, and create an annotated `vX.Y.Z` tag only after all gates pass.
8. **Push only with authorization** — Pushing `main` or a release tag requires explicit approval from the repository owner. Tag pushes trigger the Windows Release workflow and create a Draft Release.

## Updater signing

The Tauri updater requires a minisign key pair. Keep the private key and password outside the repository and store them only in the maintainer's secure secret manager and GitHub Actions secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

The public key belongs in `frontend/src-tauri/tauri.conf.json`. The configuration must keep `createUpdaterArtifacts: true` so the build emits signed update artifacts and `latest.json`. Never print, commit, or paste a private key into an issue or pull request.

## Current release outputs

The tag workflow runs on `windows-latest` and builds NSIS and MSI installers. It also uploads updater metadata when signing secrets are configured. A Draft Release is not a published release; maintainers must inspect the files, release notes, and signature-related artifacts before publishing.

## Failure handling

- If a quality gate fails, stop and fix the cause before tagging.
- If the tag workflow fails, fix the code on `main`, then delete and recreate the tag on the corrected commit. Do not publish a tag pointing to a commit that lacks the fix.
- Before any remote rollback, preserve evidence and obtain maintainer approval. Never force-push or delete a remote tag as an unreviewed workaround.

See the [full Chinese release workflow](release-workflow.md) for signing initialization, exact commands, tag recovery, and the Tauri-specific pitfalls.
