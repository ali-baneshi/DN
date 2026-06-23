# Output Formats

`dn-kernel` supports three output modes:

<<<<<<< HEAD
- text
- JSON via `--json`
- markdown via `--markdown`

Only one output mode is allowed per command.

## JSON schema v2

Top-level fields:

- `schema_version`
- `metadata`
- `stats`
- `integrations`
- `diagnostics`
- `files`
- `summary`

### `metadata`

- `root`
- `profile`
- `profile_source`
- `command`
- `output_format`
- `summary_only`
- `duration_ms`
- `truncated`

### `stats`

- `files_discovered`
- `files_scanned`
- `files_selected`
- `files_skipped`
- `total_files`
- `total_bytes`
- `skipped_large_files`
- `findings_total`
- `severity_breakdown`

### `integrations`

Contains two objects:

- `worker`
- `provider`

Each reports enablement, mode, strictness, and usage. Provider also reports `max_ai_files` and `files_sent`.

### `diagnostics`

Structured diagnostics replace the old free-form `errors` list.

Each item contains:

- `level`
- `source`
- `code`
- `message`
- `path` optional

### `files`

Each file entry contains:

- `path`
- `size`
- `language` optional
- `findings`
- `content_preview` optional
- `integration_notes` optional

Each finding contains:

- `rule`
- `severity`
- `message`
- `category` optional
- `line` optional
- `source` optional
- `origin` (`deterministic`, `worker`, `provider`)

Local deterministic findings may also include `line` when the scanner can identify a concrete source line.

## Text output

Text output is summary-oriented and stable enough for human logs, but JSON is the compatibility surface for automation.

## Markdown output

Markdown includes:

- execution summary
- integration status
- severity totals
- grouped findings by file
- diagnostics section
- empty-state handling

## Content previews

`--content` enables short previews. Treat previews as potentially secret-bearing.


## Fix command output

`dn-cli fix --json` returns a command-specific payload listing changed or changeable files, whether the run was a dry-run, and the subset of fixable rules currently supported.
=======
- default plain-text report (default)
- JSON (`--json`)
- Markdown (`--markdown`)

Only one output mode is allowed per command.

## Text output

Human-readable key/value lines:

- root
- profile / profile_source
- provider
- worker
- files counters
- severity counts
- findings count
- optional errors

## JSON output

The JSON schema is centered on `ScanReport`.

### Top-level fields

- `root`: canonical scan root
- `profile`: effective profile name
- `provider`: provider identity string
- `worker`: worker summary (`python:python`, `python:python (single-shot)`, or `disabled`)
- `profile_source`: `builtin` or `file:<path>`
- `files_discovered`: matching files before selection and skip decisions
- `files_scanned`: files actually analyzed
- `files_selected`: files included in `files` array (same as scanned)
- `files_skipped`: discovered but not scanned
- `total_files`: compatibility alias for discovered count
- `total_bytes`: bytes read from scanned files
- `skipped_large_files`: files skipped due max-file-size limit
- `truncated`: whether scan was cut off by `max_files`/`max_total_bytes`
- `errors`: diagnostics collected during scan
- `files`: per-file findings and optional `content_preview`
- `severity_breakdown`
- `duration_ms`
- `summary`

`errors` are non-fatal diagnostics (for example: worker/protocol/provider failures).

`--json` and `--markdown` are intentionally mutually exclusive.

`provider`, `worker`, `files`, `errors`, and counter fields are currently a public compatibility surface.
Avoid breaking them without documenting changes.

## Markdown output

Markdown is designed for PR comments and review notes. It includes:

- header with profile/provider/worker metadata
- counters and severity summary
- grouped findings by file
- empty state when no findings
- error list

## Content previews

Use `--content` to include `content_preview` in JSON/Markdown output for each scanned file.

Security note: previews can include secrets, tokens, or credentials. Avoid posting
raw previews to public channels.
>>>>>>> feature/persistent-workers
