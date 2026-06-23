# dn-kernel

<<<<<<< HEAD
[![Rust](https://img.shields.io/badge/Rust-CLI%20%26%20runtime-000000?logo=rust)](https://www.rust-lang.org/)
[![C](https://img.shields.io/badge/C-worker%20for%20kernel%20scans-00599C?logo=c&logoColor=white)](workers/c)
[![Python](https://img.shields.io/badge/Python-worker-3776AB?logo=python&logoColor=white)](https://www.python.org/)
[![TOML](https://img.shields.io/badge/TOML-profiles-9C4121)](https://toml.io/)
[![YAML](https://img.shields.io/badge/YAML-profiles-CB171E?logo=yaml&logoColor=white)](https://yaml.org/)
[![JSON](https://img.shields.io/badge/JSON-schemas%20%26%20protocol-5E5C5C?logo=json&logoColor=white)](https://www.json.org/json-en.html)
[![Markdown](https://img.shields.io/badge/Markdown-reporting-000000?logo=markdown)](https://www.markdownguide.org/)
[![GitHub Actions](https://img.shields.io/badge/GitHub%20Actions-CI%2Fsmoke%2Fdocs-2088FF?logo=githubactions&logoColor=white)](https://github.com/features/actions)

`dn-kernel` is a local-first repository review CLI for scanning source trees and producing structured, repeatable findings for maintainers, reviewers, and CI pipelines.

It now includes a kernel-ready C worker for Linux kernel style and safety review.

It sits between ad-hoc grep scripts and heavyweight security/review platforms: fast enough to run locally, explicit enough to trust in automation, and small enough to extend without losing control of the trust boundary.

## Technology stack

`dn-kernel` is built with and around these technologies:

- `Rust`: core CLI, runtime, scanner, diagnostics, report model, worker/provider orchestration
- `Python`: optional language-aware worker execution path
- `TOML` and `YAML`: local and inherited profile definitions
- `JSON`: machine-readable output, protocol payloads, automation contract
- `Markdown`: human-readable review reporting and handbook-style docs
- `GitHub Actions`: CI, smoke validation, and docs consistency checks

## What this tool is

`dn-kernel` is a deterministic repository inspection tool with optional integration layers.

Its core model has three analysis planes:

- deterministic local rules that always run on scanned text files
- worker-based analysis for suspicious files in supported languages
- provider-backed review for opt-in AI-style or model-driven passes

The output is designed to be:

- useful on a laptop before opening a pull request
- stable enough for CI quality gates
- explicit about failures, skips, and integration behavior
- reviewable by humans in plain text or Markdown
- consumable by tooling through schema version `2`

## What this tool is not

`dn-kernel` is intentionally not:

- a full SAST platform
- a hosted code scanning service
- an AST-complete analyzer for every supported language
- a remote-only AI wrapper
- a secret-management product

If you need deep semantic analysis, taint tracking, or enterprise policy enforcement, treat `dn-kernel` as an earlier review layer rather than a replacement.

## Why this project exists

`dn-kernel` exists to cover a practical gap between opaque AI review tools and brittle one-off scripts:

- local-first review should still be valuable without a remote dependency
- deterministic rules should remain available even when integrations are disabled
- worker and provider layers should be additive, not mandatory
- failures should surface as diagnostics instead of disappearing into best-effort behavior
- automation should rely on a versioned output contract rather than scraping logs

## Who it is for

`dn-kernel` is built for:

- maintainers preparing a first public release
- developers doing a local pre-PR pass
- reviewers auditing unfamiliar or inherited repositories
- CI pipelines that need bounded, machine-readable review signals
- security-minded teams that want a lightweight local-first review layer

## Where it is useful

### 1. Local repository review before opening a PR

Use it when you want a quick pass over a codebase before sending changes for review.
It is especially useful for catching:

- TODO markers left behind in comments
- suspicious secret-like assignments
- obvious hardcoded values
- hidden-file surprises
- profile-specific review concerns

### 2. Reviewing unfamiliar or inherited repositories

If you just cloned a new project, took ownership of an older service, or need to inspect vendor/internal code quickly, `dn-kernel` gives you a bounded first pass with clear diagnostics and file/byte limits.

### 3. CI quality gates

With `--json`, `--summary-only`, and `--fail-on`, the tool can be used as a lightweight gate in GitHub Actions or other CI systems.

### 4. Security-minded code inspection

It is not a replacement for dedicated security tooling, but it is useful as a fast review layer for suspicious patterns, secret exposure signals, unsafe usage, and repository hygiene issues.

### 5. Human-readable review artifacts

Markdown output is meant to be pasted into issues, PRs, or internal engineering discussions without post-processing.

### 6. Extensible experimentation

If you are exploring provider-backed review or worker-driven language analysis, `dn-kernel` gives you a small explicit base to build on.

## Feature overview

`dn-kernel` focuses on a few things deliberately:

- deterministic repository scanning
- `--fast` mode with worker/provider bypass and cache reuse
- content-hashed `.dn-cache` entries
- `.dn/ignore` glob support
- kernel-focused C worker with batched parallel scanning
- profile-driven behavior
- bounded analysis with explicit file and byte limits
- structured diagnostics instead of silent failure
- stable JSON contract for automation
- readable Markdown for human review
- opt-in integrations instead of hidden side effects

## Analysis model at a glance

### Deterministic local rules

These rules always run on scanned text content for the active profile. The current built-in registry covers 19 rules across four practical groups and now has practical coverage hooks for 10 common languages through the core detector plus workers:

- maintainability: `todo-comment`, `large-file`, `hard-to-read-function`, `debug-print`, `commented-out-code`, `wildcard-import`
- modernization: `deprecated-api`
- reliability: `empty-error-handler`, `network-without-timeout`, `assert-or-panic-in-production`
- security: `unsafe-usage`, `possible-secret`, `hardcoded-value`, `weak-hash-usage`, `insecure-random`, `shell-command-concatenation`, `sql-string-concatenation`, `path-traversal-join`, `dangerous-deserialization`

The local detector posture is intentionally balanced:

- obvious placeholders and examples such as `changeme`, `example`, and `${TOKEN}` are suppressed to reduce false positives
- common assignment shapes such as `=`, `:`, JSON-style keys, and single-quoted values are recognized to reduce false negatives
- findings now include line numbers when the local detector can identify a concrete source line

### Worker analysis

Worker analysis is optional and only runs when:

- the active profile enables workers
- the file content matches suspicious patterns
- the language is supported by the worker registry
- the worker executable/script is actually available

This keeps workers useful without making them a hidden dependency of every scan.

Current worker coverage now includes:

- Python worker path for Python-specific suspicious cases
- TypeScript/JavaScript worker findings for `eval`, dynamic code, DOM XSS, command injection, tainted-flow-to-sink patterns, JWT misconfiguration, and missing timeout handling
- Java worker findings for command execution, dangerous deserialization, weak hashing, SQL concatenation, empty catch blocks, missing timeouts, tainted flows, and path traversal patterns
- Go, PHP, Ruby, and Shell/Bash coverage in the deterministic core for common security and reliability smells

### Provider analysis

Provider analysis is also optional and bounded by profile settings such as:

- `ai.enabled`
- `ai.max_ai_files`
- `ai.max_content_chars`
- suspicious pattern triggers
- severity filtering
- strict vs non-strict integration behavior

Current provider states:

- `disabled`: stable
- `mock`: testing-only
- `ollama`: experimental and restricted to local endpoints such as `localhost`, `127.0.0.1`, and `::1`

## Rule registry core

The Rust runtime now includes a built-in multi-language rule registry for Rust, Python, JavaScript, TypeScript, Java, Go, PHP, Ruby, and Shell/Bash. This gives the core scanner broader coverage without requiring a worker for every language.

The current registry now includes 19 built-in deterministic rules spanning maintainability, modernization, reliability, and security use cases, with safe autofix support reserved for the narrowest low-risk cases.

## Current status

`dn-kernel` `v1.1.0` is the current stable release target and is intended to be adoptable for local use, CI experiments, and maintainers preparing or auditing repositories.

Current guarantees:

- the Rust workspace builds and key runtime/CLI tests pass before release tagging
- CLI behavior is covered by unit/integration tests
- schema version `2` is the active JSON compatibility surface
- worker and provider integrations are opt-in and explicitly reported
- GitHub workflows validate CI, smoke behavior, and docs consistency
=======
`dn-kernel` is a terminal-first repository review CLI for scanning source trees and producing structured, repeatable findings.

It is designed for developers, maintainers, security reviewers, and automation workflows that need a fast local way to inspect a codebase for architectural issues, suspicious patterns, maintainability concerns, and profile-driven review signals.

In short: **point it at a repository, choose a profile, and get a deterministic review report in plain text, JSON, or Markdown.**

---

## Who this is for

`dn-kernel` is useful for:

- **developers** who want a quick local review before opening a PR
- **maintainers** who want a consistent way to inspect repositories
- **security-minded teams** who want lightweight scans for suspicious patterns and risky usage
- **tooling and automation workflows** that need machine-readable review output
- **experimentation and extension work** where optional workers or provider-backed review flows may be added later

If you want a local-first CLI that is predictable, scriptable, and transparent about its limits, this project is built for that use case.

---

## What it does

`dn-kernel` scans a repository or directory and generates structured findings based on built-in or custom profiles.

Core capabilities include:

- deterministic repository scanning
- built-in review profiles such as:
  - `quick`
  - `security`
  - `architecture`
  - `deep`
  - `performance`
  - `maintainability`
  - `strict`
  - and other specialized profiles
- local custom profiles from `.dn/profiles/<name>.toml|yml|yaml`
- plain text output for terminal use
- JSON output for automation and pipelines
- Markdown output for human-readable review reports
- optional hidden-file scanning with `--hidden`
- optional content previews with `--content`
- optional extension points for:
  - Python worker-based analysis
  - provider-backed review flows (`disabled`, `mock`, current `ollama` scaffold)

The project aims to keep scanning behavior predictable and explicit. When optional integrations fail, errors are reported in diagnostics instead of crashing the CLI.

---

## Typical use cases

Common ways to use `dn-kernel` include:

- checking a repository before review or merge
- running a quick local security-oriented scan
- generating JSON output for CI or internal tooling
- producing Markdown review artifacts for discussion
- experimenting with custom profile rules for a specific codebase
- inspecting hidden paths such as `.github`, `.env`, or other dotfiles when needed

---
>>>>>>> feature/persistent-workers

## Quick start

### Build from source
<<<<<<< HEAD

```bash
cargo build --workspace
make -C workers/c
cargo run -p dn-cli -- scan . --profile quick
```

### Install locally

```bash
cargo install --path apps/dn-cli
dn-cli scan . --profile quick
```

## Kernel scanning

Use the built-in `kernel-c` profile to scan Linux kernel subtrees locally:

```bash
dn-cli scan ~/src/linux --profile kernel-c --json
dn-cli scan ~/src/linux/drivers --profile kernel-c --fast
```

`--fast` keeps the run deterministic and local-only by skipping worker and provider analysis while still using the content-hashed cache.

## Common commands

```bash
dn-cli scan . --profile quick
dn-cli scan . --profile security --json --fail-on medium
dn-cli review . --profile architecture --markdown --content
dn-cli profiles list . --json
dn-cli profiles show quick . --json
dn-cli validate-profile examples/profiles/my-security.toml . --json
dn-cli doctor . --json
dn-cli rules --json
dn-cli fix . --profile quick --dry-run --json
sh tests/c/run.sh
```

## Command surface

Primary commands:

- `scan <path>`
- `review <path>`
- `profiles list <root>`
- `profiles show <name-or-path> <root>`
- `validate-profile <path> <root>`
- `doctor <root>`
- `rules`
- `fix <path>`

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
- `1`: runtime, configuration, or scan execution failure
- `2`: scan succeeded but `--fail-on` threshold was reached
- `3`: `validate-profile` or `doctor` detected a command-level failure

## JSON schema v2

`--json` emits a versioned report with this top-level shape:

- `schema_version`
- `metadata`
- `stats`
- `integrations`
- `diagnostics`
- `files`
- `summary`

Key notes:

- `metadata` captures the command, root, profile, source, format, truncation, and summary-only mode
- `stats` captures file counters, total bytes, skipped-large-file counts, and severity totals
- `integrations` reports worker/provider enablement, strictness, usage, and limits
- `diagnostics` is the structured channel for warnings and errors
- `files` contains per-file findings, language hints, optional previews, and integration notes
- local findings may include `line` when the scanner could identify a specific source line

For the exact contract, see `docs/output.md` and `docs/compatibility.md`.

## Profiles

Built-in profiles include:
=======
```bash
cargo build --workspace
cargo run -p dn-cli -- scan . --profile quick

### Install locally

bash
cargo install --path apps/dn-cli
dn-cli scan . --profile quick

### Example commands

bash
dn-cli scan . --profile quick
dn-cli scan . --profile security --json
dn-cli scan . --profile architecture --markdown
dn-cli scan . --profile my-security --hidden --content
dn-cli review . --profile architecture --json

`review` is an alias of `scan`.

---

## What the output looks like

`dn-kernel` supports three practical output styles:

- **plain text** for direct terminal use
- **JSON** for machine-readable automation
- **Markdown** for readable reports and sharing

Examples:

bash
dn-cli scan . --profile quick
dn-cli scan . --profile security --json
dn-cli scan . --profile architecture --markdown

Only one output mode can be selected per command.

---

## Profiles

Profiles define how a scan behaves: which rules are active, what files are considered, and which limits or integrations apply.

Built-in profile examples include:
>>>>>>> feature/persistent-workers

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

<<<<<<< HEAD
Local profiles are resolved from `<scan-root>/.dn/profiles/<name>.toml|yml|yaml`.

Resolution order:

1. explicit file path passed to `--profile`
2. local profile at `<scan-root>/.dn/profiles/<name>.toml|yml|yaml`
3. built-in profile

### Which profile should I use?

- `quick`: default local pass while iterating
- `security`: balanced secret/safety-oriented review with worker/provider hooks
- `pre-merge`: bounded CI gate with a conservative automation posture
- `production-readiness`: stronger first-release pass for maintainers
- `legacy-modernization`: broader cleanup pass for inherited or older repositories
- `strict`: stronger threshold and broader integrated review when you want more noise in exchange for more scrutiny

### Tracked starter profiles

Tracked examples live under `examples/profiles/` and are meant to be copied into `.dn/profiles/` and adjusted:

- `ci-fast.toml`: compact CI gate with AI disabled
- `my-security.toml`: balanced security-focused local review
- `maintainer-review.toml`: maintainer-oriented release pass with worker/provider support
- `legacy-audit.toml`: broader legacy cleanup and modernization sweep

## Custom profile workflow

Fastest path for a repository-specific profile:

1. copy the closest file from `examples/profiles/`
2. place it at `<scan-root>/.dn/profiles/<name>.toml`
3. run `dn-cli validate-profile <path> <root>`
4. run `dn-cli profiles show <name> <root> --json`
5. tune suspicious triggers, limits, and integration settings for your repository

Rule of thumb:

- add more suspicious patterns when you want worker/provider paths to trigger more often
- reduce suspicious patterns when integrations are firing on too much irrelevant content
- keep deterministic rules small and explicit for stable CI behavior
- prefer inheritance over duplicating whole profile bodies

## Trust boundaries and security model

Important hardening choices in this project:

- repository contents are treated as untrusted input
- symlinks are not followed during scanning
- profile names and inheritance paths reject traversal-like values
- profile inheritance depth is bounded
- worker/provider failures surface as diagnostics instead of silently disappearing
- AI/provider responses are bounded and sanitized before becoming findings
- `--content` is opt-in because it can surface secrets in output
- secret-like local rules suppress obvious placeholders, examples, and env indirection patterns
- current `ollama` support is intentionally limited to local endpoints

For deeper detail, see `docs/threat-model.md`.

## Distribution and installation

The repository now includes release/distribution scaffolding for:

- binary release archives via `.github/workflows/release.yml`
- a starter Homebrew formula at `packaging/homebrew/dn-kernel.rb`
- an official composite GitHub Action at `.github/actions/dn-kernel`

These files are release-ready scaffolding, but Homebrew SHA256 values must be filled from real tagged release artifacts before public package publication.

## Safe autofix

`dn-cli fix <path>` is intentionally conservative. In this version it supports low-risk cleanup for:

- `todo-comment`
- `debug-print`
- `commented-out-code`
- `wildcard-import` (rewritten as a review marker rather than silently expanded)

The deliberately small fix surface is a safety choice: rules that could change behavior, semantics, or incident visibility remain report-only until they can be proven safe across languages and repository styles.

Use `--dry-run` first to preview changes.

## Current limitations and non-goals

You should be aware of these boundaries when adopting `dn-kernel`:

- rule coverage is intentionally small and explicit, not exhaustive
- suspicious trigger quality still depends on profile tuning for your repository
- worker coverage depends on supported languages and available runtimes
- provider-backed review is bounded and optional, not the source of truth
- `dn-kernel` should complement, not replace, specialized security tooling

## CI and automation use

A typical CI-oriented command:

```bash
dn-cli scan . --profile pre-merge --json --summary-only --fail-on medium
```

This gives a stable schema, compact logs, and a non-zero exit when findings cross the configured threshold.

## Documentation map

Core docs:

- `docs/cli.md`
- `docs/output.md`
- `docs/profiles.md`
- `docs/scanner.md`
- `docs/providers.md`
- `docs/architecture.md`
- `docs/troubleshooting.md`
- `docs/compatibility.md`
- `docs/threat-model.md`
- `docs/development.md`
- `docs/action.md`
- `docs/fixes.md`
- `docs/distribution.md`

Project process and metadata:

- `CONTRIBUTING.md`
- `SECURITY.md`
- `SUPPORT.md`
- `RELEASE.md`
- `CHANGELOG.md`
- `ROADMAP.md`
=======
You can also add local custom profiles under:

text
<scan-root>/.dn/profiles/

Resolution order is:

1. explicit profile path passed via `--profile`
2. local profile under `<scan-root>/.dn/profiles/<name>.toml|yml|yaml`
3. built-in profile

Example custom profile:

toml
name = "my-security"
inherits = "security"
include_hidden = true

[rules]
deterministic_rules = ["todo-comment", "possible-secret", "unsafe-usage"]
suspicious_patterns = ["password", "api_key", "token"]
prioritize = ["possible-secret", "unsafe-usage"]
min_severity = "info"

[file_selection]
include_binary = false
include_hidden = true
exclude_globs = [".git/**", "target/**"]

[limits]
max_file_size_bytes = 2_097_152
max_file_read_bytes = 16_384
max_total_bytes = 50_000_000
max_files = 10_000

[worker]
enabled = true

[ai]
enabled = true
max_ai_files = 30
max_content_chars = 1000
provider = { type = "mock", message = "Explain top risks briefly" }

For more detail, see [`docs/profiles.md`](docs/profiles.md).

---

## Hidden files and content previews

By default, hidden files and directories are excluded.

Use:

bash
dn-cli scan . --profile quick --hidden

to include entries such as:

- `.env`
- `.github`
- other dotfiles and hidden directories

You can also enable file content previews:

bash
dn-cli scan . --profile quick --content

This includes a limited preview in scan results.

> Warning: `--content` can expose secrets or other sensitive material in output. Use it carefully, especially in shared environments or public logs.

---

## Worker and provider integrations

`dn-kernel` has two optional extension layers:

- **worker layer**: external language-aware analysis
  - **Python worker**: AST-based analysis for Python code
  - **C worker**: Linux kernel coding style checks (checkpatch.pl rules)
    - Offline, no network dependencies
    - Implements 5 key kernel style rules (line-length, space-before-tab, trailing-whitespace, keyword-spacing, brace-style)
- **provider layer**: AI-style or provider-backed review flows

Current provider states include:

- `disabled`
- `mock`
- `ollama` scaffold

These integrations are optional. If they are unavailable or fail, the CLI continues and reports diagnostics in the output instead of terminating unexpectedly.

---

## Docker

A CLI-focused Docker image is included for quick evaluation:

bash
docker build -t dn-kernel -f docker/Dockerfile .
docker run --rm -v "$PWD":/workspace -w /workspace dn-kernel scan /workspace --profile quick --json

Notes:

- the default image is focused on the Rust CLI runtime
- Python worker support is **not installed** in the default image
- `--python-worker` may therefore report integration or startup diagnostics unless you build a custom image with Python and worker dependencies included

Current status:

- Docker configuration is present and intended for CLI usage
- full end-to-end Docker validation has **not** been confirmed in this environment due to external registry/network TLS timeout issues

---

## Installation requirements

To build or run from source, you typically need:

- Rust toolchain and `cargo`
- `git` for repository-root/profile workflows
- Python 3 only if you want Python worker-based analysis

---

## Design goals

This project is built around a few practical goals:

- **predictable local behavior**
- **deterministic scanning**
- **stable, script-friendly output**
- **clear failure reporting**
- **optional integrations instead of mandatory remote dependencies**

That means the CLI prefers explicit diagnostics over silent behavior, and integration failures are surfaced in reports rather than hidden.

---

## Current project status

`dn-kernel` is currently **pre-release**.

What that means in practice:

- core CLI behavior is usable
- profile-driven local scanning is the main focus
- documentation and behavior are being tightened for public use
- some optional integration paths are still evolving
- Docker validation is not fully confirmed in this environment
- `ollama` support is scaffolded, not production-hardened

If you are evaluating the project today, it is best treated as an early public release aimed at real usage, feedback, and iteration.

---

## Documentation

Project documentation:

- [`docs/cli.md`](docs/cli.md)
- [`docs/architecture.md`](docs/architecture.md)
- [`docs/scanner.md`](docs/scanner.md)
- [`docs/profiles.md`](docs/profiles.md)
- [`docs/providers.md`](docs/providers.md)
- [`docs/output.md`](docs/output.md)
- [`docs/protocol.md`](docs/protocol.md)
- [`docs/development.md`](docs/development.md)
- [`docs/troubleshooting.md`](docs/troubleshooting.md)

Project metadata and process docs:

- [`CHANGELOG.md`](CHANGELOG.md)
- [`SECURITY.md`](SECURITY.md)
- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`ROADMAP.md`](ROADMAP.md)

---

## Quick troubleshooting

- **Unknown profile**: ensure the profile exists under `<scan root>/.dn/profiles` or pass an explicit file path
- **Worker appears inactive**: check JSON or Markdown output for `worker` and `errors`; worker analysis only runs in supported cases
- **Hidden files not showing up**: verify the scan root and check ignore rules or `exclude_globs`
- **Unexpected output differences**: confirm you are using the intended profile and only one output mode flag

---
>>>>>>> feature/persistent-workers

## Contributing

Contributions are welcome.

<<<<<<< HEAD
Before changing CLI behavior or report shape:

- update tests in the same change set
- update docs in the same change set
- record user-visible changes in `CHANGELOG.md`
- document compatibility impact in `docs/compatibility.md`

## Security reporting

If you believe you found a security issue, follow `SECURITY.md` instead of filing a public issue immediately.
=======
Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) before changing public CLI behavior, docs, or profile semantics.

A good default contribution style is:

- keep behavior and documentation aligned
- prefer explicit error handling
- add or update tests when changing public-facing behavior

---

## Security

If you believe you have found a security issue, please see [`SECURITY.md`](SECURITY.md) for reporting guidance.

---

## Roadmap and limitations

Current limitations include:

- provider calls are synchronous and profile-driven
- worker protocol currently centers on Python implementation
- `ollama` integration is scaffolded and not yet production-hardened

Planned next steps are tracked in [`ROADMAP.md`](ROADMAP.md).
>>>>>>> feature/persistent-workers
