# Scanner

`dn-kernel` scans repositories with predictable policy:

<<<<<<< HEAD
- include/exclude selection
- file and byte limits
=======
- file selection via include/exclude patterns
- size/resource limits
>>>>>>> feature/persistent-workers
- deterministic local rules
- optional worker analysis
- optional provider review

## File discovery

Scanning starts at the target root and uses ignore-aware traversal.
<<<<<<< HEAD
Hidden paths are excluded by default; use `--hidden` to include them.

Default excluded globs include:

- `.git/**`
- `target/**`
- `node_modules/**`

## Counters and semantics

- `files_discovered`: files that passed include/exclude globs
- `files_scanned`: files actually analyzed
- `files_selected`: files emitted in public report output
- `files_skipped`: discovered files not analyzed
- `total_files`: compatibility alias for discovered count
- `skipped_large_files`: files skipped by size limit
- `truncated`: scan stopped because file or byte limits were reached

## Limits

Profiles and CLI overrides can define:
=======
Hidden paths are excluded by default; use `--hidden` to include dotfiles and dot-directories.

Rules:

- `ignore` handles `.gitignore` semantics.
- `WalkBuilder.hidden` is toggled by effective `include_hidden`.
- default excluded globs are `target/**`, `node_modules/**`, `.git/**`.

## Counters and semantics

- `files_discovered`: files that passed include/exclude globs.
- `files_scanned`: files that were analyzed.
- `files_selected`: files emitted in `files` output (equivalent to scanned).
- `files_skipped`: discovered files that were not scanned.
- `total_files`: compatibility alias of discovered count.
- `skipped_large_files`: files skipped because `max_file_size_bytes` was exceeded.
- `truncated`: true when scan terminated due to `max_files` or `max_total_bytes` policy.

## Limits

Per-profile or CLI-overridden limits:
>>>>>>> feature/persistent-workers

- `max_file_size_bytes`
- `max_file_read_bytes`
- `max_total_bytes`
- `max_files`

<<<<<<< HEAD
`--max-files` overrides the profile limit for the current run.

## Deterministic vs suspicious analysis

- deterministic rules always run on scanned text content
- worker/provider integrations only run when suspicious patterns match
- provider usage is bounded by `ai.max_ai_files` and content-size limits
- provider findings may be filtered by `ai.min_severity`

Current local rule behavior is intentionally conservative:

- secret-like rules suppress obvious placeholders such as `changeme`, `example`, `dummy`, and env-indirection values like `${TOKEN}`
- secret-like rules still recognize common config shapes such as `key = "..."`, `key: '...'`, and JSON-style `"key": "..."` assignments
- worker/provider execution is gated by suspicious patterns so teams can widen or narrow coverage through profiles


## Built-in rule library

The current built-in deterministic registry includes 19 rules grouped into:

- maintainability
- modernization
- reliability
- security

This is still intentionally much smaller than large external ecosystems such as Semgrep, but it is broad enough to cover common first-pass repository risks across Rust, Python, JavaScript, TypeScript, Java, Go, PHP, Ruby, and Shell/Bash.

The current priority is quality and signal density rather than raw rule count. New rules should land with:

- explicit category and severity
- line-aware findings whenever possible
- language-conscious matching
- tests demonstrating both detection and likely false-positive suppression

## Summary-only mode

`--summary-only` preserves metadata, stats, integrations, and diagnostics while emitting an empty public `files` array in JSON output.
This is useful for compact CI logs and artifact summaries.
=======
These are enforced before expensive analysis.

## Content preview

`--content` (or profile output settings) adds short `content_preview` fields to report entries.
By default previews are omitted.

Security note: previews can leak secrets; this is expected and flagged in docs/help.

## Deterministic vs suspicious checks

- Deterministic rules always run against scanned file content (for supported text files).
- Suspicious checks are pattern-based; worker/provider are only attempted when `suspicious_patterns` matches.
- provider calls are bounded by `ai.max_ai_files` and content-length.
>>>>>>> feature/persistent-workers
