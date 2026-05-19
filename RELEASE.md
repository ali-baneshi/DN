# Release Guide

Before tagging a release:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p dn-cli -- scan . --profile quick --json --summary-only
cargo run -p dn-cli -- review . --profile architecture --markdown
```

Checklist:

- `CHANGELOG.md` updated
- `README.md` and `docs/` synced with behavior
- JSON schema version reviewed
- exit code behavior reviewed
- workflow health green on GitHub
