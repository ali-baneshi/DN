# Changelog

## [Unreleased]

## [1.0.0] - 2026-05-19

### Added

- Expand practical deterministic coverage across Rust, Python, JavaScript, TypeScript, Java, Go, PHP, Ruby, and Shell/Bash.
- Deepen Java and TypeScript worker coverage for higher-signal repository review.
- Add safe autofix support for more low-risk cleanup findings.
- Add GitHub-oriented release, docs, and community scaffolding for open-source maintenance.

### Changed

- Stabilize the repository around schema version `2` and structured diagnostics as the main automation surface.
- Reduce false positives in local rules while improving common false-negative paths for security and reliability findings.
- Clarify adoption, compatibility, and operational guidance for first-time open-source users.

### Notes

- `ollama` remains experimental and local-only by design.
- Worker and provider integrations remain opt-in; deterministic scanning remains the default trust base.


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
