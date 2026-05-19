#!/usr/bin/env bash
set -Eeuo pipefail

if [[ -f .env ]]; then
  set -a
  source .env
  set +a
fi

cargo run -p dn-cli -- review "${1:-.}"
