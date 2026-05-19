# Changelog

## [Unreleased]

### Added

- Expand the deterministic multi-language rule registry to 19 built-in rules in the Rust core runtime.
- Add `dn-cli rules` for inspecting the rule registry.
- Add `dn-cli fix` with safe autofix support for `todo-comment` and `debug-print`.
- Add release/distribution scaffolding: release workflow, Homebrew formula starter, and official composite GitHub Action.
- Add Java and TypeScript worker entrypoints alongside the existing Python worker path.

- Add schema v2 report contract with `metadata`, `stats`, `integrations`, `diagnostics`, and per-finding `origin`.
- Add CLI subcommands `profiles list`, `profiles show`, `validate-profile`, and `doctor`.
- Add scan flags `--fail-on`, `--summary-only`, `--strict-integrations`, and `--max-files`.
- Add community/release files: `SUPPORT.md`, `RELEASE.md`, `CODEOWNERS`, `docs/compatibility.md`, and `docs/threat-model.md`.
- Add GitHub workflows for smoke validation and docs consistency checks.
- Add more tracked profile templates under `examples/profiles/` for CI, maintainer review, and legacy audit workflows.

### Changed

- Replace free-form runtime `errors` output with structured `diagnostics` in the public JSON contract.
- Improve Markdown output to include execution summary, integration status, and diagnostics.
- Strengthen profile validation and integration diagnostics for open-source usage.
- Reduce secret-detection false positives by suppressing obvious placeholders, examples, comments, and env-indirection patterns.
- Improve false-negative coverage for secret-like findings across `=`, `:`, JSON-style, and single-quoted assignments.
- Restrict `ollama` provider endpoints to local-only targets and validate worker protocol versions more strictly.

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
