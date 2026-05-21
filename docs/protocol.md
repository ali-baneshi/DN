# Worker Protocol

Workers communicate with the runtime through a simple protocol.

The Python worker uses newline-delimited JSON over stdio.
The runtime uses the shared request/response shape from `crates/dn-ipc`:

- request: `WorkerRequest`
- response: `WorkerResponse`

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

`status` may be `error` with an `error` field.

## Session flow

1. Runtime starts worker process `python workers/python/dn_worker.py`.
2. Runtime sends one `hello` request once per process lifecycle.
3. For each suspicious file, runtime sends `analyze_file` requests.
4. Worker parses each request and writes back one JSON response per request.

## C Worker Protocol

The C worker uses a simpler command-line interface:

### Invocation

```bash
./dn-worker-c <file.c>
```

### Output format

```json
{
  "file": "example.c",
  "issues": [
    {
      "rule": "line-length",
      "severity": "warning",
      "message": "Line exceeds 80 characters (95 characters)",
      "line": 10,
      "column": 1
    }
  ]
}
```

## Versioning and compatibility

Current protocol version is `0.1.0`.

The runtime validates `request_id` for each worker response to avoid cross-request drift.

## Failure modes

- Worker spawn failure (`python` binary missing, bad script path): captured as diagnostic and scan continues.
- Malformed worker response JSON: diagnostic is recorded and analysis continues.
- Worker request error: diagnostic is recorded and scan continues.
- If multiple worker attempts fail, the request is retried according to profile retry policy.

The scanner intentionally prefers scan completion over hard failure when worker communication breaks.
