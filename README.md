# dn-kernel

[![Rust](https://img.shields.io/badge/Rust-CLI%20%26%20runtime-000000?logo=rust)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-worker-3776AB?logo=python&logoColor=white)](https://www.python.org/)
[![TOML](https://img.shields.io/badge/TOML-profiles-9C4121)](https://toml.io/)
[![YAML](https://img.shields.io/badge/YAML-profiles-CB171E?logo=yaml&logoColor=white)](https://yaml.org/)
[![JSON](https://img.shields.io/badge/JSON-schemas%20%26%20protocol-5E5C5C?logo=json&logoColor=white)](https://www.json.org/json-en.html)
[![Markdown](https://img.shields.io/badge/Markdown-reporting-000000?logo=markdown)](https://www.markdownguide.org/)
[![GitHub Actions](https://img.shields.io/badge/GitHub%20Actions-CI%2Fsmoke%2Fdocs-2088FF?logo=githubactions&logoColor=white)](https://github.com/features/actions)

`dn-kernel` is a terminal-first repository review CLI for scanning source trees and producing structured, repeatable findings.

It is built for developers, maintainers, security reviewers, and automation workflows that need a fast local way to inspect a codebase for suspicious patterns, maintainability risks, architecture smells, and profile-driven review signals.

In short: point it at a repository, choose a profile, and get a deterministic review report in text, JSON, or Markdown.

## Technology stack

`dn-kernel` is built with and around these technologies:

- `Rust`: core CLI, runtime, scanning engine, diagnostics model, protocol integration
- `Python`: optional worker-based language analysis extension
- `TOML` and `YAML`: local and built-in profile configuration formats
- `JSON`: machine-readable output, worker/provider protocol payloads, schema-oriented automation
- `Markdown`: human-readable review reports and documentation
- `GitHub Actions`: CI, smoke validation, and docs consistency checks

## What this tool is

`dn-kernel` is a local-first code review and repository inspection tool.
It is not a general-purpose SAST platform, not a hosted code scanning service, and not a remote-only AI wrapper.

Its core value is combining three things in one predictable CLI:

- deterministic local scanning
- optional language-aware worker analysis
- optional provider-backed review flows

That makes it useful when you want a review artifact that is:

- fast to generate
- explicit about what happened
- inspectable in CI
- readable by humans
- stable enough for automation

## Why this project exists

`dn-kernel` exists to cover a gap between opaque AI review tools and brittle ad-hoc scripts:

- local-first review should remain useful without a remote service
- deterministic rules should always be available
- provider and worker integrations should be additive, not mandatory
- failures should surface as diagnostics rather than hidden best-effort behavior
- machine-readable output should be a first-class contract, not an afterthought

## Where it is useful

`dn-kernel` is designed for several practical situations:

### 1. Local repository review before opening a PR

Use it when you want a quick pass over a codebase before sending changes for review.
It is especially useful for catching:

- TODO markers left behind
- suspicious secret-like assignments
- obvious hardcoded values
- hidden-file surprises
- profile-specific review concerns

### 2. Reviewing unfamiliar or inherited repositories

If you just cloned a new project, took ownership of an older service, or need to inspect vendor/internal code quickly, `dn-kernel` gives you a bounded first pass with clear diagnostics and limits.

### 3. CI quality gates

With `--json`, `--summary-only`, and `--fail-on`, the tool can be used as a lightweight quality gate in GitHub Actions or other CI systems.

### 4. Security-minded code inspection

It is not a replacement for dedicated security tooling, but it is very useful as a fast review layer for suspicious patterns, secret exposure signals, and repository hygiene concerns.

### 5. Human-readable review artifacts

Markdown output makes it easy to generate review notes that can be pasted into issues, PRs, or internal engineering discussions.

### 6. Extensible experimentation

If you are exploring provider-backed code review or worker-driven language analysis, `dn-kernel` provides a small and explicit base to build on.

## What it does well

`dn-kernel` focuses on a few things deliberately:

- deterministic repository scanning
- profile-driven behavior
- bounded analysis with explicit limits
- structured diagnostics instead of silent failure
- stable JSON contract for automation
- readable markdown for human review
- opt-in integrations rather than hidden side effects

## Current status

`dn-kernel` is currently pre-release.

Current guarantees:

- Rust workspace builds, tests, formats, and passes clippy cleanly
- CLI supports review-local and CI-oriented workflows
- schema version `2` is the current JSON contract
- provider and worker integrations are opt-in and explicitly reported
- GitHub workflows validate CI, smoke behavior, and docs consistency

## Quick start

### Build from source

```bash
cargo build --workspace
cargo run -p dn-cli -- scan . --profile quick
```

### Install locally

```bash
cargo install --path apps/dn-cli
dn-cli scan . --profile quick
```

### Common commands

```bash
dn-cli scan . --profile quick
dn-cli scan . --profile security --json --fail-on medium
dn-cli review . --profile architecture --markdown --content
dn-cli profiles list . --json
dn-cli profiles show quick . --json
dn-cli validate-profile examples/profiles/my-security.toml . --json
dn-cli doctor . --json
```

## CLI capabilities

Primary commands:

- `scan <path>`
- `review <path>`
- `profiles list <root>`
- `profiles show <name-or-path> <root>`
- `validate-profile <path> <root>`
- `doctor <root>`

Useful flags for `scan` and `review`:

- `--profile <name|path>`
- `--json`
- `--markdown`
- `--content`
- `--hidden`
- `--python-worker`
- `--fail-on <none|info|low|medium|high|critical>`
- `--summary-only`
- `--strict-integrations`
- `--max-files <n>`

## Exit codes

`dn-cli` uses these exit codes:

- `0`: command succeeded and quality threshold was not tripped
- `1`: runtime/configuration/scan execution failure
- `2`: scan succeeded but `--fail-on` threshold was reached
- `3`: validation or doctor command failed

## JSON schema v2

`--json` emits a versioned report with this top-level shape:

- `schema_version`
- `metadata`
- `stats`
- `integrations`
- `diagnostics`
- `files`
- `summary`

`metadata` captures execution context.

`stats` captures counters and severity totals.

`integrations` reports provider and worker activation, usage, strictness, and limits.

`diagnostics` is a structured list of warnings/errors instead of free-form strings.

`files` contains per-file findings, language hints, optional previews, and integration notes.

For the exact contract, see `docs/output.md` and `docs/compatibility.md`.

## Profiles

Built-in profiles include:

- `quick`
- `security`
- `architecture`
- `deep`
- `performance`
- `maintainability`
- `ai-generated-code-review`
- `legacy-modernization`
- `pre-merge`
- `strict`
- `educational`
- `production-readiness`

Local profiles are resolved from `<scan-root>/.dn/profiles/<name>.toml|yml|yaml`.

Resolution order:

1. explicit file path passed to `--profile`
2. local profile at `<scan-root>/.dn/profiles/<name>.toml|yml|yaml`
3. built-in profile

Tracked example profiles for experimentation live under `examples/profiles/`.
To use one as a scan-root local profile, copy it into `.dn/profiles/` inside the repository you want to scan.

Starter profiles included in `examples/profiles/`:

- `ci-fast.toml`: compact CI gate with AI disabled
- `my-security.toml`: balanced security-focused local review
- `maintainer-review.toml`: maintainer-oriented pass with worker/provider support
- `legacy-audit.toml`: broader legacy cleanup and modernization sweep

## Security model and trust boundaries

Important hardening choices in this project:

- repository contents are treated as untrusted input
- symlinks are not followed during scanning
- profile names and inheritance paths reject traversal-like values
- profile inheritance depth is bounded
- worker and provider failures are surfaced as diagnostics
- AI/provider responses are bounded and sanitized before becoming findings
- `--content` remains opt-in because it can surface secrets in output
- secret-like local rules suppress obvious placeholders, examples, and env-indirection patterns to cut noisy false positives
- secret-like local rules recognize common `=`, `:`, JSON, YAML, and quoted assignment styles to improve practical coverage

For details, see `docs/threat-model.md`.

## Provider and worker integrations

`dn-kernel` has two optional extension layers:

- worker layer: language-aware analysis via external workers
- provider layer: AI-style or provider-backed review

Current provider status:

- `disabled`: stable
- `mock`: testing-only
- `ollama`: experimental

The default posture remains local-first. Remote or local provider-backed review is opt-in through profiles.
For safety, the current `ollama` path is intentionally restricted to local endpoints such as `localhost` / `127.0.0.1`.

## CI and automation use

A typical CI-oriented command:

```bash
dn-cli scan . --profile quick --json --summary-only --fail-on medium
```

This gives a stable schema, short logs, and non-zero exit when findings cross the configured threshold.

## Documentation map

- `docs/cli.md`
- `docs/output.md`
- `docs/profiles.md`
- `docs/providers.md`
- `docs/architecture.md`
- `docs/troubleshooting.md`
- `docs/compatibility.md`
- `docs/threat-model.md`
- `docs/development.md`

Project process and metadata:

- `CONTRIBUTING.md`
- `SECURITY.md`
- `SUPPORT.md`
- `RELEASE.md`
- `CHANGELOG.md`
- `ROADMAP.md`

## Contributing

Contributions are welcome.

Before changing CLI behavior or report shape:

- update tests in the same change set
- update docs in the same change set
- record user-visible changes in `CHANGELOG.md`
- document compatibility impact in `docs/compatibility.md`

## Security reporting

If you believe you found a security issue, follow `SECURITY.md` instead of filing a public issue immediately.
