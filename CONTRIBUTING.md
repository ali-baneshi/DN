# Contributing to dn-kernel

Thanks for considering a contribution.

`dn-kernel` is intentionally small: CLI + runtime + IPC + optional Python worker.
The public quality bar is predictable behavior, clear diagnostics, stable automation contracts, and aligned documentation.

## Project structure

- `apps/dn-cli`: command-line UX, rendering, and exit-code behavior
- `crates/dn-runtime`: profile loading, validation, scanning, diagnostics, report generation
- `crates/dn-ipc`: worker request/response protocol models
- `workers/python`: optional Python worker implementation
- `docs/`: public operator, contributor, and compatibility documentation

## Contribution expectations

When changing user-visible behavior:

- update tests in the same patch
- update docs in the same patch
- update `CHANGELOG.md`
- update `docs/compatibility.md` if CLI flags, exit codes, or JSON contract changed

## Development setup

```bash
cargo build --workspace
cargo test --workspace
```

Recommended validation loop:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Useful smoke checks

```bash
cargo run -p dn-cli -- scan . --profile quick --json --summary-only
cargo run -p dn-cli -- scan . --profile security --json --fail-on medium
cargo run -p dn-cli -- review . --profile architecture --markdown --content
cargo run -p dn-cli -- profiles list . --json
cargo run -p dn-cli -- doctor . --json
```

## Python worker development

```bash
cd workers/python
python -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements.txt
```

## Code standards

- keep behavior deterministic where practical
- avoid panics for user-facing errors
- prefer structured diagnostics over opaque string failures
- preserve local-first behavior by default
- keep worker/provider integrations opt-in and explicit
- add regression coverage before merging behavior changes

## Communication

A strong issue or PR includes:

- exact command and flags used
- profile name/path
- whether behavior is local-review, CI-gating, or both
- expected vs actual behavior
- JSON payload or markdown output when relevant
- compatibility impact, if any
