# Output Formats

`dn-kernel` supports three output modes:

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
