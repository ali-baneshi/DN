# Profiles

`dn-kernel` uses profile-driven policy to keep scans predictable and team-friendly.
A profile controls what is scanned, how deep the analysis goes, and whether worker/provider integrations are enabled.

## Profile sources and discovery

Profiles are resolved in this order:

1. explicit file path
2. scan-root local profile at `<root>/.dn/profiles/<name>.toml|yml|yaml`
3. built-in profile

`<root>` is the first argument of `scan` or `review`.

## Built-in profiles

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

## Local profile format

Profiles are TOML or YAML files with these sections:

- `name` (required)
- `inherits` (optional)
- `[rules]`
- `[file_selection]`
- `[limits]`
- `[worker]`
- `[ai]`
- `[output]`

```toml
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
min_severity = "info"
strict = false
include_summary_note = true
provider = { type = "mock", message = "Explain top risks briefly" }
```

`include_hidden` is accepted at top-level for compatibility and is equivalent to `file_selection.include_hidden`.

Tracked example profiles are available under `examples/profiles/` for validation and experimentation.
If you want the scanner to resolve one by local profile name, copy it into `<scan-root>/.dn/profiles/`.

## Validation rules

A profile is rejected when:

- `name` is empty
- `limits.max_files = 0`
- `limits.max_total_bytes = 0`
- `limits.max_file_read_bytes = 0`
- traversal-like values are used in profile names or inheritance names

A profile may still be valid but emit diagnostics when:

- `ai.enabled = true` while provider is `disabled`
- worker analysis is enabled without suspicious patterns
- non-standard severity strings are normalized

## Inheritance and merge behavior

- scalar values override when non-zero or explicitly set
- list fields override when non-empty
- `include_binary` and `include_hidden` are additive
- builtin profiles can be inherited by local profiles

## Practical recipes

### CI quality gate

```toml
name = "pre-merge-fast"
inherits = "pre-merge"
[limits]
max_files = 2500
[ai]
enabled = false
```

### Strict provider use

```toml
name = "provider-strict"
inherits = "security"
[ai]
enabled = true
strict = true
provider = { type = "mock", message = "Summarize high risk findings only" }
```
