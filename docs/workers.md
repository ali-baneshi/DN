# Workers

`dn-kernel` workers are local executables that read one JSON request from stdin and write one JSON response to stdout.

## C worker protocol

- request: `WorkerRequest`
- response: `WorkerResponse`
- schema version: `2`

Example request:

```json
{"schema_version":"2","protocol_version":"1.0.0","request_id":"scan-1","method":"analyze_file","params":{"path":"drivers/char/foo.c","language":"c","content":"..."}}
```

Batch request:

```json
{"schema_version":"2","protocol_version":"1.0.0","request_id":"scan-batch-1","method":"scan_files","params":{"files":[{"path":"drivers/foo.c","language":"c","content":"..."},{"path":"kernel/bar.c","language":"c","content":"..."}]}}
```

Example response:

```json
{"schema_version":"2","protocol_version":"1.0.0","request_id":"scan-1","status":"ok","findings":[{"rule":"printk-without-level","severity":"warning","message":"printk should include a KERN_ level","line":12,"column":1,"category":"style"}]}
```

Batch response:

```json
{"schema_version":"2","protocol_version":"1.0.0","request_id":"scan-batch-1","status":"ok","results":[{"path":"drivers/foo.c","findings":[{"rule":"return-without-unlock","severity":"high","message":"Return occurs while a lock is still held","line":44,"column":2,"category":"concurrency"}]}]}
```

## Parallel scanning

- `scan_files` accepts a batch of C files.
- The worker uses a `pthread` pool with up to 4 threads.
- The Rust runtime switches to batch mode for queued C files when `--fast` is off and enough suspicious files accumulate.

## Build and test

```bash
make -C workers/c
sh tests/c/run.sh
```

## Adding a rule

1. Add semantic detection in `workers/c/rules.c`.
2. Emit `rule`, `severity`, `message`, `line`, `column`, and `category`.
3. Add positive or negative fixtures in `tests/c/`.
4. Update the golden JSON and verify with `sh tests/c/run.sh`.
