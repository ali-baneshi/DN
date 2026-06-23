# Provider Model

`dn-kernel` separates two analysis planes:

<<<<<<< HEAD
- worker layer: external language-aware analysis
- provider layer: optional AI-style review

## Provider configuration

Provider configuration lives under `[ai]` in a profile:
=======
- **Worker layer**: language-aware analysis via external worker processes (currently Python)
- **Provider layer**: optional AI-style review via pluggable providers

## Provider configuration

Provider configuration is under `[ai]` in a profile:
>>>>>>> feature/persistent-workers

```toml
[ai]
enabled = true
max_ai_files = 30
max_content_chars = 2048
<<<<<<< HEAD
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

For safety, `ollama` base URLs are currently restricted to local endpoints (`localhost`, `127.0.0.1`, `::1`).

## Strict vs non-strict behavior

- default behavior: provider failures are emitted as diagnostics and scan continues
- `ai.strict = true` or `--strict-integrations`: provider failures become hard failures

## Safety notes

- repository content is sent as untrusted data, never executable instructions
- provider responses are bounded, normalized, and sanitized before findings are emitted
- use explicit profiles when enabling provider review in shared environments
=======
provider = { type = "mock", message = "Explain risky patterns." }
```

## Provider implementations

### `disabled`

Default state. No AI review calls are made.

### `mock`

Current safe default for deterministic behavior. Mock provider returns a single synthetic finding
(`mock-ai-review`) with the configured message.

### `ollama` (future-ready)

Scaffold exists to call a local chat completion endpoint in the style expected by
`/api/chat/completions`.

The implementation requires:

- `base_url`
- `model`
- optional `api_key`, `timeout_secs`, `temperature`
- optional `extra_system_prompt`

This path is intentionally explicit and isolated so local providers can be added with a minimal rewrite.

## Provider status in reports

`provider` in the report is emitted as `<provider>@<profile_source>`
(e.g. `mock@builtin` or `ollama@builtin` / `disabled@file:...`).

## Provider troubleshooting

- If AI fails for a file, scan continues and the error is collected under `errors`.
- If no provider is enabled, no synthetic AI findings are added.
- Use `--json`/`--markdown` to inspect provider state and diagnostics.

>>>>>>> feature/persistent-workers
