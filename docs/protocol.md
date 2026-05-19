# Worker Protocol

The Python worker protocol is newline-delimited JSON over stdio.
The runtime uses the shared request/response shape from `crates/dn-ipc`.

## Request shape

```json
{
  "protocol_version": "0.1.0",
  "request_id": "scan-1",
  "method": "analyze_file",
  "params": {
    "path": "src/main.rs",
    "language": "rust",
    "content": "..."
  }
}
```

## Response shape

```json
{
  "protocol_version": "0.1.0",
  "request_id": "scan-1",
  "status": "ok",
  "findings": [
    {
      "severity": "high",
      "rule": "python-eval-usage",
      "message": "Use of eval() detected",
      "line": 12,
      "category": "security"
    }
  ]
}
```

## Session flow

1. runtime starts the worker process
2. runtime sends one `hello` request per worker lifecycle
3. runtime sends `analyze_file` for suspicious files
4. worker returns one response per request

## Runtime behavior

- worker findings become report findings with `origin = "worker"`
- worker failures are emitted as structured diagnostics
- `--strict-integrations` can convert worker failures to hard scan failures
- request IDs are validated to avoid cross-request drift

## Failure modes

- worker spawn failure
- malformed worker response JSON
- empty worker response
- request/response ID mismatch
- explicit worker `error` status

By default the scanner prefers completion with diagnostics over hard failure.
