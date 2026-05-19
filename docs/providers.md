# Provider Model

`dn-kernel` separates two analysis planes:

- worker layer: external language-aware analysis
- provider layer: optional AI-style review

## Provider configuration

Provider configuration lives under `[ai]` in a profile:

```toml
[ai]
enabled = true
max_ai_files = 30
max_content_chars = 2048
min_severity = "info"
strict = false
include_summary_note = true
provider = { type = "mock", message = "Explain risky patterns." }
```

## Provider states

### `disabled` — stable

No provider review is attempted.

### `mock` — testing-only

Returns deterministic synthetic findings for testing and examples.

### `ollama` — experimental

Calls a local chat-completions style endpoint.

Expected fields:

- `base_url`
- `model`
- optional `api_key`
- optional `timeout_secs`
- optional `temperature`
- optional `extra_system_prompt`

## Strict vs non-strict behavior

- default behavior: provider failures are emitted as diagnostics and scan continues
- `ai.strict = true` or `--strict-integrations`: provider failures become hard failures

## Safety notes

- repository content is sent as untrusted data, never executable instructions
- provider responses are bounded, normalized, and sanitized before findings are emitted
- use explicit profiles when enabling provider review in shared environments
