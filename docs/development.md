# Development Guide

## Local setup

```bash
cargo build --workspace
cargo test --workspace
```

## Recommended validation sequence

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Useful command workflows

```bash
cargo run -p dn-cli -- scan . --profile quick
cargo run -p dn-cli -- scan . --profile quick --json --summary-only
cargo run -p dn-cli -- scan . --profile security --json --fail-on medium
cargo run -p dn-cli -- review . --profile architecture --markdown --content
cargo run -p dn-cli -- profiles list . --json
cargo run -p dn-cli -- profiles show quick . --json
cargo run -p dn-cli -- doctor . --json
cargo run -p dn-cli -- rules --json
cargo run -p dn-cli -- fix . --profile quick --dry-run --json
```

## Testing guidance

If you change runtime/profile behavior, add or update tests in:

- `crates/dn-runtime/src/lib.rs`
- `apps/dn-cli/tests/cli.rs`

If you change JSON shape, markdown rendering, or exit codes:

- update docs in the same patch
- update `CHANGELOG.md`
- review `docs/compatibility.md`

## Python worker development

```bash
cd workers/python
python -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements.txt
python dn_worker/__main__.py
```

## Release-oriented checks

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p dn-cli -- scan . --profile quick --json --summary-only
cargo run -p dn-cli -- review . --profile architecture --markdown
```


## Rule registry development

When adding new built-in rules:

- add the rule to `crates/dn-runtime/src/rules.rs` with a `RuleSpec` entry
- keep the rule language-aware when possible
- prefer findings with explicit line numbers
- add regression tests for both positive detection and obvious false-positive suppression
- avoid adding autofix support unless the change is safely local and behavior-preserving
