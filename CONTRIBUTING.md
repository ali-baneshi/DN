# Contributing to dn-kernel

Thanks for considering a contribution.

<<<<<<< HEAD
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
=======
dn-kernel is a small ecosystem project (CLI + runtime + IPC + optional Python worker).  
The priority is predictable behavior and clear diagnostics over speed hacks.

## Project structure

- `apps/dn-cli`: terminal command surface and UX
- `crates/dn-runtime`: scanning runtime, profile resolution, report model
- `crates/dn-ipc`: worker protocol schema
- `workers/python`: optional Python worker implementation

## Scope and release expectations

- The codebase targets a pre-release CLI utility; behavior should stay stable across releases.
- Public command-line and report fields are treated as a compatibility surface.
- API changes should be accompanied by migration notes in docs.
- Keep `CHANGELOG.md` updated for behavior changes visible to users.
>>>>>>> feature/persistent-workers

## Development setup

```bash
<<<<<<< HEAD
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
=======
cargo build
cargo test --workspace
```

If you change runtime behavior:

- run formatting/tests first (`cargo fmt --all`, `cargo test --workspace`)
- run smoke checks in `docs/cli.md`
- update docs in one commit with behavior changes

For Python worker development:
>>>>>>> feature/persistent-workers

```bash
cd workers/python
python -m venv .venv
. .venv/bin/activate
<<<<<<< HEAD
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
=======
pip install -r requirements.txt
```

## Before submitting

- run format/tests:

```bash
cargo fmt --all
cargo test --workspace
```

- keep output and docs aligned with behavior
- include tests for new CLI behavior when changing user-facing behavior
- avoid changing public reporting fields without migration notes
- keep behavior deterministic where possible
- prefer profile-driven behavior over hidden command behavior
- add unit tests for library logic and CLI integration tests for user-impacting changes

## Code style

- keep behavior deterministic
- avoid panics on user errors
- return actionable diagnostics in `errors` and CLI stderr
- prefer profile-driven behavior over command-line flags where practical
- keep comments minimal and only where behavior is not obvious

## Tests

- unit tests under `crates/*/src/lib.rs` for core logic
- integration CLI tests under `apps/dn-cli/tests/cli.rs`
- add real-world scenario tests for flags that affect scan shape (`--hidden`, custom profile loading, `--json`/`--markdown`, worker paths)
- run focused end-to-end manual checks when touching user flow:
  - `cargo run -p dn-cli -- scan . --profile quick --json`
  - `cargo run -p dn-cli -- review . --profile architecture --json`
  - `cargo run -p dn-cli -- scan . --profile security --markdown`
- update docs for any report-field or user-visible wording changes in the same PR

## Communication

Open issues/PRs with: expected behavior, command used to reproduce, and any relevant output snippets.

## Maintainer checklist

- [ ] behavior change has tests (unit + integration as needed)
- [ ] CLI/runtime docs updated
- [ ] protocol/provider assumptions validated
- [ ] changelog/notes updated for release-relevant behavior
>>>>>>> feature/persistent-workers
