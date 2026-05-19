# Scanner

`dn-kernel` scans repositories with predictable policy:

- include/exclude selection
- file and byte limits
- deterministic local rules
- optional worker analysis
- optional provider review

## File discovery

Scanning starts at the target root and uses ignore-aware traversal.
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

- `max_file_size_bytes`
- `max_file_read_bytes`
- `max_total_bytes`
- `max_files`

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

## Summary-only mode

`--summary-only` preserves metadata, stats, integrations, and diagnostics while emitting an empty public `files` array in JSON output.
This is useful for compact CI logs and artifact summaries.
