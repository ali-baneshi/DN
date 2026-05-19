#!/usr/bin/env bash
set -Eeuo pipefail

if [[ -f .env ]]; then
  set -a
  source .env
  set +a
fi

cargo run -p dn-cli -- review .
JOB="$(cat .dn/jobs/latest_job_id)"
test -f ".dn/jobs/$JOB/request.json"
test -f ".dn/jobs/$JOB/response.json"
test -f ".dn/jobs/$JOB/output/report.md"
test -f ".dn/jobs/$JOB/output/report.json"
echo "smoke ok: $JOB"
