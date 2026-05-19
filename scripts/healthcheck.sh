#!/usr/bin/env bash
set -euo pipefail

echo "[1/4] building workspace..."
cargo build --workspace >/dev/null

echo "[2/4] running tests..."
cargo test >/dev/null

echo "[3/4] checking python worker startup..."
python -m workers.python.dn_worker >/tmp/dn_worker_ready.json &
worker_pid=$!
sleep 1
kill "$worker_pid" 2>/dev/null || true

echo "[4/4] running scan with python worker..."
cargo run -p dn-cli -- scan . --python-worker >/tmp/dn_health_scan.txt

grep -q "errors=0" /tmp/dn_health_scan.txt

echo "healthcheck: ok"
