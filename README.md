# dn-kernel

`dn-kernel` is a terminal-first repository review CLI for scanning source trees and producing structured, repeatable findings.

It is built for developers, maintainers, security reviewers, and automation workflows that need a fast local way to inspect a codebase for suspicious patterns, maintainability risks, architecture smells, and profile-driven review signals.

In short: point it at a repository, choose a profile, and get a deterministic review report in text, JSON, or Markdown.

## Why this project exists

`dn-kernel` exists to cover a gap between opaque AI review tools and brittle ad-hoc scripts:

- local-first review should remain useful without a remote service
- deterministic rules should always be available
- provider and worker integrations should be additive, not mandatory
- failures should surface as diagnostics rather than hidden best-effort behavior

## Current status

`dn-kernel` is currently pre-release.

Current guarantees:

- Rust workspace builds, tests, formats, and passes clippy cleanly
- CLI supports review-local and CI-oriented workflows
- schema version `2` is the current JSON contract
- provider and worker integrations are opt-in and explicitly reported

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
dn-cli validate-profile .dn/profiles/custom.toml . --json
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

## Security model and trust boundaries

Important hardening choices in this project:

- repository contents are treated as untrusted input
- symlinks are not followed during scanning
- profile names and inheritance paths reject traversal-like values
- profile inheritance depth is bounded
- worker and provider failures are surfaced as diagnostics
- AI/provider responses are bounded and sanitized before becoming findings
- `--content` remains opt-in because it can surface secrets in output

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
