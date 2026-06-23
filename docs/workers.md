# Workers

<<<<<<< HEAD
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
=======
Workers are language-specific code analyzers that extend DN Kernel's capabilities beyond basic pattern matching. Each worker implements specialized analysis for a particular programming language or domain.

## Overview

Workers communicate with DN Kernel through a simple JSON-based protocol:
- Input: File path as command-line argument
- Output: JSON with structured issues

## Available Workers

### Python Worker

**Location:** `workers/python/`

**Purpose:** Python-specific code analysis

**Features:**
- AST-based analysis
- Import checking
- Code complexity metrics
- Style violations

**Usage:**
```bash
cd workers/python
python -m dn_worker <file.py>
```

### C Worker

**Location:** `workers/c/`

**Purpose:** Linux kernel coding style enforcement

**Features:**
- Offline analysis (no network dependencies)
- Fast text-based parsing
- Implements checkpatch.pl rules

**Rules Implemented:**

1. **line-length** (warning)
   - Lines should not exceed 80 characters
   - Helps maintain readability on various displays

2. **space-before-tab** (error)
   - No spaces before tabs in indentation
   - Prevents mixed indentation issues

3. **trailing-whitespace** (warning)
   - No trailing whitespace at end of lines
   - Keeps diffs clean

4. **keyword-spacing** (warning)
   - Space required after keywords: `if`, `for`, `while`, `switch`
   - Example: `if (x)` not `if(x)`

5. **brace-style** (warning)
   - Opening braces on same line as statement (K&R style)
   - Example: `if (x) {` not `if (x)\n{`

**Building:**
```bash
cd workers/c
make
```

**Usage:**
```bash
./dn-worker-c <file.c>
```

**Example Output:**
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
    },
    {
      "rule": "space-before-tab",
      "severity": "error",
      "message": "Space before tab in indentation",
      "line": 15,
      "column": 2
    }
  ]
}
```

## Worker Protocol

All workers must follow this protocol:

### Input
- Single command-line argument: file path to analyze
- File content is read by the worker

### Output
- JSON object to stdout
- Required fields:
  - `file`: string (path to analyzed file)
  - `issues`: array of issue objects

### Issue Object Format
```json
{
  "rule": "rule-name",
  "severity": "error|warning|info",
  "message": "Human-readable description",
  "line": 10,
  "column": 5
}
```

### Exit Codes
- `0`: Success (even if issues found)
- `1`: Error (file not found, parse error, etc.)

## Integration with Profiles

Workers are configured in profile files:

```toml
[profile]
name = "kernel-c"
language = "c"

[workers]
c = { enabled = true, path = "workers/c/dn-worker-c" }

[rules]
[rules.line-length]
enabled = true
severity = "warning"

[rules.space-before-tab]
enabled = true
severity = "error"
```

## Creating a New Worker

To create a new worker:

1. **Choose a directory structure:**
   ```
   workers/<language>/
   ├── README.md
   ├── <worker-binary>
   └── tests/
   ```

2. **Implement the protocol:**
   - Accept file path as argument
   - Parse/analyze the file
   - Output JSON with issues

3. **Add a profile:**
   ```
   profiles/<language>.toml
   ```

4. **Document:**
   - Add to this file
   - Create worker-specific README
   - Add examples

### Example Worker (Bash)

```bash
#!/bin/bash
# workers/bash/dn-worker-bash

FILE="$1"

if [ ! -f "$FILE" ]; then
    echo '{"file":"'$FILE'","issues":[]}' >&2
    exit 1
fi

# Simple check: lines over 80 chars
echo '{"file":"'$FILE'","issues":['

first=true
line_num=0
while IFS= read -r line; do
    line_num=$((line_num + 1))
    if [ ${#line} -gt 80 ]; then
        [ "$first" = false ] && echo ","
        echo '{"rule":"line-length","severity":"warning","message":"Line exceeds 80 characters","line":'$line_num',"column":1}'
        first=false
    fi
done < "$FILE"

echo ']}'
```

## Performance Considerations

- Workers should be fast (< 1 second for typical files)
- Avoid network calls (keep analysis offline)
- Handle large files gracefully
- Use streaming/incremental parsing when possible

## Testing Workers

Test your worker with various inputs:

```bash
# Normal file
./dn-worker-c test.c

# Empty file
touch empty.c
./dn-worker-c empty.c

# Non-existent file
./dn-worker-c nonexistent.c

# Large file
./dn-worker-c large_file.c
```

## Troubleshooting

### Worker not found
- Check the path in profile configuration
- Ensure worker binary is executable: `chmod +x workers/c/dn-worker-c`

### Invalid JSON output
- Test worker directly: `./dn-worker-c test.c | jq`
- Check for stderr contamination

### Worker crashes
- Test with minimal input
- Check for memory issues with large files
- Add error handling for edge cases

## Future Enhancements

Planned improvements:
- Tree-sitter integration for C worker (AST-based analysis)
- Rust worker
- Go worker
- JavaScript/TypeScript worker
- Parallel worker execution
- Worker caching
- Incremental analysis

## See Also

- [Profiles Documentation](profiles.md)
- [Architecture Documentation](architecture.md)
- [Protocol Documentation](protocol.md)
>>>>>>> feature/persistent-workers
