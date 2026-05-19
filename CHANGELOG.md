# Changelog

## [Unreleased]

### Added

- Add schema v2 report contract with `metadata`, `stats`, `integrations`, `diagnostics`, and per-finding `origin`.
- Add CLI subcommands `profiles list`, `profiles show`, `validate-profile`, and `doctor`.
- Add scan flags `--fail-on`, `--summary-only`, `--strict-integrations`, and `--max-files`.
- Add community/release files: `SUPPORT.md`, `RELEASE.md`, `CODEOWNERS`, `docs/compatibility.md`, and `docs/threat-model.md`.
- Add GitHub workflows for smoke validation and docs consistency checks.

### Changed

- Replace free-form runtime `errors` output with structured `diagnostics` in the public JSON contract.
- Improve Markdown output to include execution summary, integration status, and diagnostics.
- Strengthen profile validation and integration diagnostics for open-source usage.

### Notes

- JSON automation consumers should migrate to schema version `2`.
- `ollama` remains experimental; `mock` remains testing-only.

## [0.1.0] - 2026-05-14

- Add initial CLI runtime with profile-driven repository scanning.
- Add built-in profiles and local profile loading with inheritance.
- Add optional Python worker integration and provider abstraction.
- Add deterministic/local + suspicious/AI-style provider pipeline.
- Add plain-text, JSON, and Markdown output formats.
- Add robust tests for hidden files, profile loading, worker path, and errors.
