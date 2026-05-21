# Development Guide

## Local setup

```bash
cargo build --workspace
cargo test --workspace
```

If you are working on profile/runtime behavior, add tests under:

- `crates/dn-runtime/src/lib.rs` unit tests for profile loading and scan semantics.
- `apps/dn-cli/tests/cli.rs` integration tests for CLI-facing behavior.

Use trusted local fixtures when testing profile behavior; avoid loading unknown profiles from untrusted sources in CI.

## Useful command workflows

- Format: `cargo fmt --all`
- Test: `cargo test --workspace`
- Build: `cargo build --workspace`
- CLI quick smoke checks:
  - `cargo run -p dn-cli -- scan . --profile quick`
  - `cargo run -p dn-cli -- scan . --profile security --json`
- `cargo run -p dn-cli -- review . --profile architecture --markdown`
- `cargo run -p dn-cli -- scan . --profile security --json --content`
- `cargo run -p dn-cli -- scan . --profile my-security --hidden`
- `docker build -t dn-kernel -f docker/Dockerfile .`
- `docker run --rm -v "$PWD":/workspace -w /workspace dn-kernel scan /workspace --profile quick --json`

If Docker commands fail here due external registry TLS/network timeouts, record it as an environment issue and proceed with
CLI/runtime validation from source first; this is an infrastructural limitation, not a confirmed regression.

## Python worker development

```bash
cd workers/python
python -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements.txt
python dn_worker/__main__.py
```

## C worker development

```bash
cd workers/c
make
./dn-worker-c test_sample.c
```

To clean and rebuild:
```bash
make clean && make
```
## Optional helpers

- `make setup` creates a virtualenv and installs worker dependencies.
- `make scan` and `make scan-json` run sample scans over the repository.
- `scripts/healthcheck.sh` can be used for basic runtime smoke checks.

## Release checks (recommended before tagging)

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p dn-cli -- scan . --profile quick --json`
- `cargo run -p dn-cli -- scan . --profile security --markdown`
- `cargo run -p dn-cli -- review . --profile architecture --json`
- `rg -n "unknown profile|Docker readiness|TLS" README.md docs/*.md` (quick docs-path consistency check).
- `CHANGELOG.md` updated for user-visible changes
- docs and command examples reviewed for behavior alignment
- version intentionally bumped for release
- Docker release-readiness should remain blocked if build/run cannot be reproducibly validated in your environment.
