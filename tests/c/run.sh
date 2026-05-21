#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)"
WORKER="$ROOT/workers/c/dn-worker-c"

make -C "$ROOT/workers/c" >/dev/null

run_case() {
  name="$1"
  fixture="$2"
  expected="$3"
  payload=$(python3 - "$fixture" "$name" <<'PY'
from pathlib import Path
import json
import sys
fixture = Path(sys.argv[1])
request_id = sys.argv[2]
print(json.dumps({
    "schema_version": "2",
    "protocol_version": "1.0.0",
    "request_id": request_id,
    "method": "analyze_file",
    "params": {
        "path": fixture.name,
        "language": "c",
        "content": fixture.read_text(),
    },
}))
PY
)
  actual="$(mktemp)"
  printf '%s\n' "$payload" | "$WORKER" | jq --sort-keys . >"$actual"
  diff -u "$expected" "$actual"
  rm -f "$actual"
}

run_batch() {
  expected="$ROOT/tests/c/expected_batch.json"
  payload=$(python3 - "$ROOT/tests/c" <<'PY'
from pathlib import Path
import json
import sys
root = Path(sys.argv[1])
files = []
for name in ["goto_no_cleanup.c", "return_without_unlock.c", "negative_clean.c", "rcu_missing_annotation.c"]:
    path = root / name
    files.append({
        "path": path.name,
        "language": "c",
        "content": path.read_text(),
    })
print(json.dumps({
    "schema_version": "2",
    "protocol_version": "1.0.0",
    "request_id": "batch-cases",
    "method": "scan_files",
    "params": {"files": files},
}))
PY
)
  actual="$(mktemp)"
  printf '%s\n' "$payload" | "$WORKER" | jq --sort-keys . >"$actual"
  diff -u "$expected" "$actual"
  rm -f "$actual"
}

run_case "goto-no-cleanup" "$ROOT/tests/c/goto_no_cleanup.c" "$ROOT/tests/c/expected_goto_no_cleanup.json"
run_case "return-without-unlock" "$ROOT/tests/c/return_without_unlock.c" "$ROOT/tests/c/expected_return_without_unlock.json"
run_case "null-deref" "$ROOT/tests/c/null_deref_before_check.c" "$ROOT/tests/c/expected_null_deref_before_check.json"
run_case "init-exit" "$ROOT/tests/c/missing_init_exit.c" "$ROOT/tests/c/expected_missing_init_exit.json"
run_case "rcu" "$ROOT/tests/c/rcu_missing_annotation.c" "$ROOT/tests/c/expected_rcu_missing_annotation.json"
run_case "sleeping-in-atomic" "$ROOT/tests/c/sleeping_in_atomic.c" "$ROOT/tests/c/expected_sleeping_in_atomic.json"
run_case "negative-clean" "$ROOT/tests/c/negative_clean.c" "$ROOT/tests/c/expected_negative_clean.json"
run_batch
